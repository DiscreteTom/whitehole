use criterion::{criterion_group, criterion_main, Criterion};
use in_str::in_str;
use std::{fs::read_to_string, sync::LazyLock};
use whitehole::{
  action::Action,
  combinator::{eat, next, recur, wrap, Combinator},
  parser::Parser,
};

pub fn build_parser_with_recur(s: &str) -> Parser<impl Action<Value = ()>, &str> {
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

  Parser::builder().entry(ws() | value()).build(s)
}

pub fn build_parser_with_static(s: &str) -> Parser<impl Action<Value = ()>, &str> {
  // To re-use a combinator for multiple times, instead of wrapping the combinator in an Rc,
  // use a function to generate the combinator for better runtime performance (via inlining).
  fn ws() -> Combinator<impl Action<Value = ()>> {
    next(in_str!(" \t\r\n")) * (1..)
  }

  fn wso() -> Combinator<impl Action<Value = ()>> {
    ws().optional()
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
    let escape = {
      let simple = next(in_str!("\"\\/bfnrt"));
      let hex = eat('u') + next(|c| c.is_ascii_hexdigit()) * 4;
      eat('\\') + (simple | hex)
    };
    let non_escape =
      next(|c| c != '"' && c != '\\' && matches!(c, '\u{0020}'..='\u{10ffff}')) * (1..);
    let body_optional = (escape | non_escape) * ..;
    eat('"') + body_optional + '"'
  }

  fn sep() -> Combinator<impl Action<Value = ()>> {
    eat(',') + wso()
  }

  fn array() -> Combinator<impl Action<Value = ()>> {
    eat('[') + wso() + ((value() + wso()) * (..)).sep(sep()) + ']'
  }

  fn object() -> Combinator<impl Action<Value = ()>> {
    let object_item = string() + wso() + eat(':') + wso() + value();
    eat('{') + wso() + ((object_item + wso()) * (..)).sep(sep()) + '}'
  }

  // `value` will indirectly recurse to itself, so we need special treatment.
  // Use `LazyLock` to create a static `Action` implementor,
  // use `Box<dyn>` to prevent recursive/infinite type.
  fn value() -> Combinator<impl Action<Value = ()>> {
    static VALUE: LazyLock<Box<dyn Action<Value = ()> + Send + Sync>> = LazyLock::new(|| {
      Box::new(array() | object() | number() | string() | "true" | "false" | "null")
    });
    wrap(|input| VALUE.exec(input))
  }

  Parser::builder().entry(ws() | value()).build(s)
}

fn parse_json(mut parser: Parser<impl Action, &str>) {
  loop {
    let output = parser.parse();
    if output.is_none() {
      break;
    }
    // println!("{:?}", output);
  }

  if !parser.instant().rest().is_empty() {
    panic!(
      "parser failed to consume the whole input, remaining: {:?}",
      &parser.instant().rest()[..100.min(parser.instant().rest().len())]
    );
  }
}

fn bench_parse(c: &mut Criterion) {
  // json files are from https://github.com/miloyip/nativejson-benchmark/tree/478d5727c2a4048e835a29c65adecc7d795360d5/data
  // you may need to download them manually
  let citm_catalog = read_to_string("bench_data/citm_catalog.json").unwrap();
  let twitter = read_to_string("bench_data/twitter.json").unwrap();
  let canada = read_to_string("bench_data/canada.json").unwrap();

  let total_bytes = citm_catalog.len() + twitter.len() + canada.len();

  c.bench_function(
    &format!(
      "json_parser_with_recur: parse 3 json files (total {} bytes)",
      total_bytes
    ),
    |b| {
      b.iter(|| {
        parse_json(build_parser_with_recur(&citm_catalog));
        parse_json(build_parser_with_recur(&twitter));
        parse_json(build_parser_with_recur(&canada));
      })
    },
  );

  c.bench_function(
    &format!(
      "json_parser_with_static: parse 3 json files (total {} bytes)",
      total_bytes
    ),
    |b| {
      b.iter(|| {
        parse_json(build_parser_with_static(&citm_catalog));
        parse_json(build_parser_with_static(&twitter));
        parse_json(build_parser_with_static(&canada));
      })
    },
  );
}

criterion_group! {
  name = benches;
  config = Criterion::default();
  targets = bench_parse
}
criterion_main!(benches);
