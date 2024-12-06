use criterion::{criterion_group, criterion_main, Criterion};
use in_str::in_str;
use std::{cell::OnceCell, fs::read_to_string, rc::Rc, sync::LazyLock};
use whitehole::{
  combinator::{eat, next, wrap},
  parse::Parse,
  parser::{Builder, Parser},
  Combinator,
};

pub fn build_parser_with_inter_mut(s: &str) -> Parser<impl Parse> {
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
  let value_rc: Rc<OnceCell<Box<dyn Parse<Kind = (), State = (), Heap = ()>>>> =
    Rc::new(OnceCell::new());
  let value = || {
    let value_rc = value_rc.clone();
    // SAFETY: we will initialize `value_rc` later before calling this closure.
    wrap(move |input| unsafe { value_rc.get().unwrap_unchecked() }.parse(input))
  };

  // Now we can use `value` in `array` and `object`.
  let array = || {
    eat('[')
      + ws().optional()
      + ((value() + ws().optional()) * (.., eat(',') + ws().optional())).optional()
      + ']'
  };
  let object = || {
    let object_item = string() + ws().optional() + eat(':') + ws().optional() + value();
    eat('{')
      + ws().optional()
      + ((object_item + ws().optional()) * (.., eat(',') + ws().optional())).optional()
      + '}'
  };

  // Finally, init `value` with `array` and `object`.
  value_rc
    .set(Box::new(wrap({
      let parser = array() | object() | number() | string() | "true" | "false" | "null";
      move |input| parser.parse(input)
    })))
    .ok();

  Builder::new().entry(ws() | value()).build(s)
}

pub fn build_parser_with_static(s: &str) -> Parser<impl Parse> {
  // To re-use a combinator for multiple times, instead of wrapping the combinator in an Rc,
  // use a function to generate the combinator for better runtime performance (via inlining).
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

  fn array() -> Combinator!() {
    eat('[')
      + ws().optional()
      + ((value() + ws().optional()) * (.., eat(',') + ws().optional())).optional()
      + ']'
  }

  fn object() -> Combinator!() {
    let object_item = string() + ws().optional() + eat(':') + ws().optional() + value();
    eat('{')
      + ws().optional()
      + ((object_item + ws().optional()) * (.., eat(',') + ws().optional())).optional()
      + '}'
  }

  // `value` will indirectly recurse to itself, so we need special treatment.
  // Use `LazyLock` to create a static `Parse` implementor,
  // use `Box<dyn>` to prevent recursive/infinite type.
  fn value() -> Combinator!() {
    static VALUE: LazyLock<Box<dyn Parse<Kind = (), State = (), Heap = ()> + Send + Sync>> =
      LazyLock::new(|| {
        Box::new(array() | object() | number() | string() | "true" | "false" | "null")
      });
    wrap(|input| VALUE.parse(input))
  }

  Builder::new().entry(ws() | value()).build(s)
}

fn parse_json(mut parser: Parser<impl Parse>) {
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
      "json_parser_with_inter_mut: parse 3 json files (total {} bytes)",
      total_bytes
    ),
    |b| {
      b.iter(|| {
        parse_json(build_parser_with_inter_mut(&citm_catalog));
        parse_json(build_parser_with_inter_mut(&twitter));
        parse_json(build_parser_with_inter_mut(&canada));
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
