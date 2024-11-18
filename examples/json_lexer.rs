use whitehole::{
  combinator::{eat, next},
  in_str,
  parse::Parse,
  parser::{Builder, Parser},
};

fn build_lexer(s: &str) -> Parser<impl Parse<Kind = ()>> {
  Builder::new()
    .entry(|b| {
      // use `b.next` instead of `next` for better type inference
      let whitespaces = b.next(in_str!(" \t\r\n")) * (1..); // repeat one or more times
      let number = {
        let digit_1_to_9 = next(|c| matches!(c, '1'..='9'));
        // use a closure to generate combinator for better performance (via inlining), instead of wrapping the combinator in an Rc
        let digits = || next(|c| c.is_ascii_digit()) * (1..);
        let integer = eat('0') | (digit_1_to_9 + digits().optional());
        let fraction = eat('.') + digits();
        let exponent = (eat('e') | 'E') + (eat('-') | '+').optional() + digits();
        eat('-').optional() + integer + fraction.optional() + exponent.optional()
      };
      let string = {
        let escape = eat('\\')
          + (next(in_str!("\"\\/bfnrt")) | (eat('u') + next(|c| c.is_ascii_hexdigit()) * 4));
        let non_escape =
          next(|c| c != '"' && c != '\\' && matches!(c, '\u{0020}'..='\u{10ffff}')) * (1..);
        let body_optional = (escape | non_escape) * ..; // repeat zero or more times
        eat('"') + body_optional + '"'
      };
      let boundary = next(in_str!("[]{}:,"));
      whitespaces | boundary | number | string | "true" | "false" | "null"
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

  let mut lexer = build_lexer(s);

  loop {
    let output = lexer.parse();
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

  let rest = lexer.instant().rest();
  if !rest.is_empty() {
    panic!(
      "lexer failed to consume the whole input, remaining: {}",
      &rest[..100.min(rest.len())]
    );
  }
}
