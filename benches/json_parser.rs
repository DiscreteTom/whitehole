use criterion::{criterion_group, criterion_main, Criterion};
use std::{cell::OnceCell, fs::read_to_string, rc::Rc};
use whitehole::{
  combinator::{eat, next, wrap},
  in_str,
  parse::Parse,
  parser::{Builder, Parser},
  Combinator,
};

fn build_parser(s: &str) -> Parser<impl Parse> {
  Builder::new()
    .entry(|b| {
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

      let array = Rc::new(OnceCell::new());
      let object_item = Rc::new(OnceCell::new());
      let object = Rc::new(OnceCell::new());
      // use dyn to prevent recursive type
      let value: Rc<dyn Parse<Kind = ()>> = Rc::new(wrap({
        let parser = rc_to_combinator(&array)
          | rc_to_combinator(&object)
          | number()
          | string()
          | "true"
          | "false"
          | "null";
        move |input| parser.parse(input)
      }));

      fn rc_to_combinator(rc: &Rc<OnceCell<Combinator!()>>) -> Combinator!() {
        wrap({
          let rc = rc.clone();
          move |input| unsafe { rc.get().unwrap_unchecked() }.parse(input)
        })
      }
      fn value_to_combinator(value: Rc<dyn Parse<Kind = ()>>) -> Combinator!() {
        wrap(move |input| value.parse(input))
      }

      array
        .set(wrap({
          let parser = eat('[')
            + ws().optional()
            + ((value_to_combinator(value.clone()) + ws().optional())
              * (.., eat(',') + ws().optional()))
              .optional()
            + ']';
          move |input| parser.parse(input)
        }))
        .ok();

      object_item
        .set(wrap({
          let parser = string()
            + ws().optional()
            + eat(':')
            + ws().optional()
            + value_to_combinator(value.clone());
          move |input| parser.parse(input)
        }))
        .ok();

      object
        .set(wrap({
          let parser = eat('{')
            + ws().optional()
            + ((rc_to_combinator(&object_item) + ws().optional())
              * (.., eat(',') + ws().optional()))
              .optional()
            + '}';
          move |input| parser.parse(input)
        }))
        .ok();

      ws() | value_to_combinator(value)
    })
    .build(s)
}

fn parse_json(s: &str) {
  let mut parser = build_parser(s);

  loop {
    let output = parser.parse();
    if output.is_none() {
      break;
    }
    // println!("{:?}", output);
  }

  if !parser.instant().rest().is_empty() {
    panic!(
      "parser failed to consume the whole input, remaining: {}",
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
      "json_parser: parse 3 json files (total {} bytes)",
      total_bytes
    ),
    |b| {
      b.iter(|| {
        parse_json(&citm_catalog);
        parse_json(&twitter);
        parse_json(&canada);
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
