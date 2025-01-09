use in_str::in_str;
use std::sync::LazyLock;
use whitehole::{
  action::Action,
  combinator::{eat, next, wrap, Combinator},
  parser::{Builder, Parser},
  range::WithRange,
};

pub fn build_parser_with_static(s: &str) -> Parser<impl Action<Value = WithRange<()>>> {
  // To re-use a combinator for multiple times, instead of wrapping the combinator in an Rc,
  // use a function to generate the combinator for better runtime performance (via inlining).
  fn ws() -> Combinator<impl Action<Value = ()>> {
    next(in_str!(" \t\r\n")) * (1..)
  }

  fn number() -> Combinator<impl Action<Value = ()>> {
    let digit_1_to_9 = next(|c| matches!(c, '1'..='9'));
    let digits = || next(|c| c.is_ascii_digit()) * (1..);
    let integer = eat('0') | (digit_1_to_9 + digits().optional());
    let fraction = eat('.') + digits();
    let exponent = (eat('e') | 'E') + (eat('-') | '+').optional() + digits();
    eat('-').optional() + integer + fraction.optional() + exponent.optional()
  }

  fn string() -> Combinator<impl Action<Value = ()>> {
    let escape =
      eat('\\') + (next(in_str!("\"\\/bfnrt")) | (eat('u') + next(|c| c.is_ascii_hexdigit()) * 4));
    let non_escape =
      next(|c| c != '"' && c != '\\' && matches!(c, '\u{0020}'..='\u{10ffff}')) * (1..);
    let body = (escape | non_escape) * ..;
    eat('"') + body.optional() + '"'
  }

  fn array() -> Combinator<impl Action<Value = ()>> {
    eat('[')
      + ws().optional()
      + ((value() + ws().optional()) * (..))
        .sep(eat(',') + ws().optional())
        .optional()
      + ']'
  }

  fn object() -> Combinator<impl Action<Value = ()>> {
    let object_item = string() + ws().optional() + eat(':') + ws().optional() + value();
    eat('{')
      + ws().optional()
      + ((object_item + ws().optional()) * (..))
        .sep(eat(',') + ws().optional())
        .optional()
      + '}'
  }

  // `value` will indirectly recurse to itself, so we need special treatment.
  // Use `LazyLock` to create a static `Parse` implementor,
  // use `Box<dyn>` to prevent recursive/infinite type.
  fn value() -> Combinator<impl Action<Value = ()>> {
    // TODO: make combinators const so we don't need LazyLock
    static VALUE: LazyLock<Box<dyn Action<Value = ()> + Send + Sync>> = LazyLock::new(|| {
      Box::new(array() | object() | number() | string() | "true" | "false" | "null")
    });
    wrap(|input| VALUE.exec(input))
  }

  Builder::new().entry((ws() | value()).range()).build(s)
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

    let mut parser = build_parser_with_static(s);

    loop {
      let output = parser.parse();
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

    let rest = parser.instant().rest();
    if !rest.is_empty() {
      panic!(
        "lexer failed to consume the whole input, remaining: {:?}",
        &rest[..100.min(rest.len())]
      );
    }
  }
}
