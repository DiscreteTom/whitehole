use in_str::in_str;
use std::sync::LazyLock;
use whitehole::{
  combinator::{eat, next, wrap},
  parse::Parse,
  parser::{Builder, Parser},
  Combinator,
};

fn ws() -> Combinator!() {
  next(in_str!(" \t\r\n")) * (1..)
}

fn number() -> Combinator!() {
  let digit_1_to_9 = next(|c| matches!(c, '1'..='9'));
  let digits = || next(|c| c.is_ascii_digit()) * (1..);
  let integer = eat('0') | (digit_1_to_9 + digits().optional());
  let fraction = eat('.') + digits();
  let exponent = (eat('e') | 'E') + (eat('-') | '+').optional() + digits();
  eat('-').optional() + integer + fraction.optional() + exponent.optional()
}

fn string() -> Combinator!() {
  let escape =
    eat('\\') + (next(in_str!("\"\\/bfnrt")) | (eat('u') + next(|c| c.is_ascii_hexdigit()) * 4));
  let non_escape =
    next(|c| c != '"' && c != '\\' && matches!(c, '\u{0020}'..='\u{10ffff}')) * (1..);
  let body = (escape | non_escape) * ..;
  eat('"') + body.optional() + '"'
}

fn value() -> Combinator!() {
  static VALUE: LazyLock<Box<dyn Parse<Kind = ()> + Send + Sync>> = LazyLock::new(|| {
    Box::new(array() | object() | number() | string() | "true" | "false" | "null")
  });
  wrap(|input| VALUE.parse(input))
}

fn array() -> Combinator!() {
  static ARRAY: LazyLock<Box<dyn Parse<Kind = ()> + Send + Sync>> = LazyLock::new(|| {
    Box::new(
      eat('[')
        + ws().optional()
        + ((value() + ws().optional()) * (.., eat(',') + ws().optional())).optional()
        + ']',
    )
  });
  wrap(|input| ARRAY.parse(input))
}

fn object_item() -> Combinator!() {
  static OBJECT_ITEM: LazyLock<Box<dyn Parse<Kind = ()> + Send + Sync>> =
    LazyLock::new(|| Box::new(string() + ws().optional() + eat(':') + ws().optional() + value()));
  wrap(|input| OBJECT_ITEM.parse(input))
}

fn object() -> Combinator!() {
  static OBJECT: LazyLock<Box<dyn Parse<Kind = ()> + Send + Sync>> = LazyLock::new(|| {
    Box::new(
      eat('{')
        + ws().optional()
        + ((object_item() + ws().optional()) * (.., eat(',') + ws().optional())).optional()
        + '}',
    )
  });
  wrap(|input| OBJECT.parse(input))
}

fn build_parser(s: &str) -> Parser<impl Parse> {
  Builder::new().entry(|_| ws() | value()).build(s)
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
