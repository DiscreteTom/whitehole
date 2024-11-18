use std::{cell::OnceCell, rc::Rc};
use whitehole::{
  combinator::{eat, next, wrap, Combinator},
  in_str,
  parse::Parse,
  parser::{Builder, Parser},
};

fn build_parser(s: &str) -> Parser<impl Parse> {
  Builder::new()
    .entry(|b| {
      // use `b.next` instead of `next` for better type inference.
      // use a closure to generate combinator for better performance (via inlining), instead of wrapping the combinator in an Rc.
      let ws = || b.next(in_str!(" \t\r\n")) * (1..);
      let number = || {
        let digit_1_to_9 = next(|c| matches!(c, '1'..='9'));
        let digits = || next(|c| c.is_ascii_digit()) * (1..);
        let integer = eat('0') | (digit_1_to_9 + digits().optional());
        let fraction = eat('.') + digits();
        let exponent = (eat('e') | 'E') + (eat('-') | '+').optional() + digits();
        eat('-').optional() + integer + fraction.optional() + exponent.optional()
      };
      let string = || {
        let escape = eat('\\')
          + (next(in_str!("\"\\/bfnrt")) | (eat('u') + next(|c| c.is_ascii_hexdigit()) * 4));
        let non_escape =
          next(|c| c != '"' && c != '\\' && matches!(c, '\u{0020}'..='\u{10ffff}')) * (1..);
        let body = (escape | non_escape) * ..;
        eat('"') + body.optional() + '"'
      };

      // for combinators that will be used recursively, we have to use a pointer with interior mutability.
      // here we use Rc<OnceCell<Combinator<_>>> to store the combinator.
      macro_rules! init_rc {
        ($name:ident, $rc_name:ident) => {
          let $rc_name: Rc<OnceCell<Combinator<_>>> = Rc::new(OnceCell::new());
          let $name = || {
            let $rc_name = $rc_name.clone();
            wrap(move |input| unsafe { $rc_name.get().unwrap_unchecked() }.parse(input))
          };
        };
      }
      init_rc!(array, array_rc);
      init_rc!(object_item, object_item_rc);
      init_rc!(object, object_rc);

      // the last recursive combinator doesn't need interior mutability
      // because we can write the closure with previous combinators.
      // use `dyn` to prevent recursive type
      let value_rc: Rc<dyn Parse<Kind = ()>> = Rc::new(wrap({
        let parser = array() | object() | number() | string() | "true" | "false" | "null";
        move |input| parser.parse(input)
      }));
      let value = || {
        let value_rc = value_rc.clone();
        wrap(move |input| value_rc.parse(input))
      };

      // set rc combinator value
      array_rc
        .set(wrap({
          let parser = eat('[')
            + ws().optional()
            + ((value() + ws().optional()) * (.., eat(',') + ws().optional())).optional()
            + ']';
          move |input| parser.parse(input)
        }))
        .ok();
      object_item_rc
        .set(wrap({
          let parser = string() + ws().optional() + eat(':') + ws().optional() + value();
          move |input| parser.parse(input)
        }))
        .ok();
      object_rc
        .set(wrap({
          let parser = eat('{')
            + ws().optional()
            + ((object_item() + ws().optional()) * (.., eat(',') + ws().optional())).optional()
            + '}';
          move |input| parser.parse(input)
        }))
        .ok();

      ws() | value()
    })
    .build(s)
}

fn main() {
  let s = r#"
    {
      "name": "John Doe",
      "age": 30,
      "is_student": false,
      "scores": [100, 90, 80],
      "address": {
        "city": "New York",
        "zip": "10001"
      }
    }
  "#;

  let mut parser = build_parser(s);

  loop {
    let output = parser.parse();
    if let Some(node) = output {
      println!(
        "{}..{}: {}",
        node.range.start,
        node.range.end,
        &s[node.range.clone()]
      );
    } else {
      break;
    }
  }

  let rest = parser.instant().rest();
  if !rest.is_empty() {
    panic!(
      "lexer failed to consume the whole input, remaining: {}",
      &rest[..100.min(rest.len())]
    );
  }
}
