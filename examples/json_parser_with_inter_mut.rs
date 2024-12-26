use in_str::in_str;
use std::{cell::OnceCell, rc::Rc};
use whitehole::{
  action::Action,
  combinator::{eat, next, wrap},
  parser::{Builder, Parser},
};

pub fn build_parser_with_inter_mut(s: &str) -> Parser<impl Action> {
  // To re-use a combinator for multiple times, instead of wrapping the combinator in an Rc,
  // use a closure to generate the combinator for better runtime performance (via inlining).
  let ws = || next(in_str!(" \t\r\n")) * (1..);
  let number = || {
    let digit_1_to_9 = next(|c| matches!(c, '1'..='9'));
    let digits = || next(|c| c.is_ascii_digit()) * (1..);
    let integer = eat('0') | (digit_1_to_9 + digits().optional());
    let fraction = eat('.') + digits();
    let exponent = (eat('e') | 'E') + (eat('-') | '+').optional() + digits();
    eat('-').optional() + integer + fraction.optional() + exponent.optional()
  };
  let string = || {
    let escape =
      eat('\\') + (next(in_str!("\"\\/bfnrt")) | (eat('u') + next(|c| c.is_ascii_hexdigit()) * 4));
    let non_escape =
      next(|c| c != '"' && c != '\\' && matches!(c, '\u{0020}'..='\u{10ffff}')) * (1..);
    let body = (escape | non_escape) * ..;
    eat('"') + body.optional() + '"'
  };

  // `value` will indirectly recurse to itself, so we need special treatment.
  // Use `Rc` to make it clone-able, use `OnceCell` to initialize it later,
  // use `Box<dyn>` to prevent recursive/infinite type.
  let value_rc: Rc<OnceCell<Box<dyn Action<Value = (), State = (), Heap = ()>>>> =
    Rc::new(OnceCell::new());
  let value = || {
    let value_rc = value_rc.clone();
    // SAFETY: we will initialize `value_rc` later before calling this closure.
    wrap(move |input| unsafe { value_rc.get().unwrap_unchecked() }.exec(input))
  };

  // Now we can use `value` in `array` and `object`.
  let array = || {
    eat('[')
      + ws().optional()
      + ((value() + ws().optional()).sep(eat(',') + ws().optional()) * (..)).optional()
      + ']'
  };
  let object = || {
    let object_item = string() + ws().optional() + eat(':') + ws().optional() + value();
    eat('{')
      + ws().optional()
      + ((object_item + ws().optional()).sep(eat(',') + ws().optional()) * (..)).optional()
      + '}'
  };

  // Finally, init `value` with `array` and `object`.
  value_rc
    .set(Box::new(wrap({
      let parser = array() | object() | number() | string() | "true" | "false" | "null";
      move |input| parser.exec(input)
    })))
    .ok();

  Builder::new().entry(ws() | value()).build(s)
}

fn main() {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_json_parser() {
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

    let mut parser = build_parser_with_inter_mut(s);

    loop {
      let output = parser.parse();
      if let Some(node) = output {
        println!(
          "{}..{}: {:?}",
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
        "lexer failed to consume the whole input, remaining: {:?}",
        &rest[..100.min(rest.len())]
      );
    }
  }
}
