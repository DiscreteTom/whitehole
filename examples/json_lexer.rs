use in_str::in_str;
use whitehole::{
  combinator::{eat, next},
  parser::{Builder, Parser},
  range::WithRange,
  A,
};

pub fn build_lexer(s: &str) -> Parser<A!(WithRange<()>)> {
  // Use `* (1..)` to repeat for one or more times.
  let whitespaces = next(in_str!(" \t\r\n")) * (1..);

  let number = {
    let digit_1_to_9 = next(|c| matches!(c, '1'..='9'));
    // To re-use a combinator for multiple times, instead of wrapping the combinator in an Rc,
    // use a closure to generate the combinator for better runtime performance (via inlining).
    let digits = || next(|c| c.is_ascii_digit()) * (1..);
    let integer = eat('0') | (digit_1_to_9 + digits().optional());
    let fraction = eat('.') + digits();
    let exponent = (eat('e') | 'E') + (eat('-') | '+').optional() + digits();
    eat('-').optional() + integer + fraction.optional() + exponent.optional()
  };

  let string = {
    let escape = {
      let simple = next(in_str!("\"\\/bfnrt"));
      let hex = eat('u') + next(|c| c.is_ascii_hexdigit()) * 4;
      eat('\\') + (simple | hex)
    };
    let non_escape =
      next(|c| c != '"' && c != '\\' && matches!(c, '\u{0020}'..='\u{10ffff}')) * (1..);
    // Use `* (..)` to repeat for zero or more times.
    let body_optional = (escape | non_escape) * ..;
    eat('"') + body_optional + '"'
  };

  let boundary = next(in_str!("[]{}:,"));

  Builder::new()
    .entry((whitespaces | boundary | number | string | "true" | "false" | "null").range())
    .build(s)
}

fn main() {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_json_lexer() {
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
          "{}..{}: {:?}",
          node.value.range.start,
          node.value.range.end,
          &s[node.value.range.clone()]
        );
      } else {
        break;
      }
    }

    let rest = lexer.instant().rest();
    if !rest.is_empty() {
      panic!(
        "lexer failed to consume the whole input, remaining: {:?}",
        &rest[..100.min(rest.len())]
      );
    }
  }
}
