use in_str::in_str;
use whitehole::{
  action::Action,
  combinator::{eat, next, recur},
  parser::Parser,
  range::WithRange,
};

pub fn build_parser_with_recur(s: &str) -> Parser<impl Action<Value = WithRange<()>>, &str> {
  // To re-use a combinator for multiple times, instead of wrapping the combinator in an Rc,
  // use a closure to generate the combinator for better runtime performance (via inlining).
  let ws = || next(in_str!(" \t\r\n")) * (1..);
  let wso = || ws().optional();
  let number = || {
    let digit_1_to_9 = next(|c| matches!(c, '1'..='9'));
    let digits = || next(|c| c.is_ascii_digit()) * (1..);
    let integer = eat('0') | (digit_1_to_9 + digits().optional());
    let fraction = eat('.') + digits();
    let exponent = (eat('e') | 'E') + (eat('-') | '+').optional() + digits();
    eat('-').optional() + integer + fraction.optional() + exponent.optional()
  };
  let string = || {
    let escape = {
      let simple = next(in_str!("\"\\/bfnrt"));
      let hex = eat('u') + next(|c| c.is_ascii_hexdigit()) * 4;
      eat('\\') + (simple | hex)
    };
    let non_escape =
      next(|c| c != '"' && c != '\\' && matches!(c, '\u{0020}'..='\u{10ffff}')) * (1..);
    let body_optional = (escape | non_escape) * ..;
    eat('"') + body_optional + '"'
  };

  // `value` will indirectly recurse to itself, so we need to use `recur` to break the cycle.
  let (value, value_setter) = recur::<_, (), (), _>();

  // We can use `value` in `array` and `object` before it is defined.
  let sep = || eat(',') + wso();
  let array = || eat('[') + wso() + ((value() + wso()) * (..)).sep(sep()) + ']';
  let object = || {
    let object_item = string() + wso() + eat(':') + wso() + value();
    eat('{') + wso() + ((object_item + wso()) * (..)).sep(sep()) + '}'
  };

  // Finally, define `value` with `array` and `object`.
  value_setter.boxed(array() | object() | number() | string() | "true" | "false" | "null");

  Parser::builder().entry((ws() | value()).range()).build(s)
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

    let mut parser = build_parser_with_recur(s);

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
