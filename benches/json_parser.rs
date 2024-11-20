use criterion::{criterion_group, criterion_main, Criterion};
use in_str::in_str;
use std::{cell::OnceCell, fs::read_to_string, rc::Rc};
use whitehole::{
  combinator::{eat, next, wrap, Combinator},
  parse::Parse,
  parser::{Builder, Parser},
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

      macro_rules! rc_combinator {
        ($name:ident, $rc_name:ident) => {
          let $rc_name: Rc<OnceCell<Combinator<_>>> = Rc::new(OnceCell::new());
          let $name = || {
            let $rc_name = $rc_name.clone();
            wrap(move |input| unsafe { $rc_name.get().unwrap_unchecked() }.parse(input))
          };
        };
      }
      rc_combinator!(array, array_rc);
      rc_combinator!(object_item, object_item_rc);
      rc_combinator!(object, object_rc);

      let value_rc: Rc<dyn Parse<Kind = ()>> = Rc::new(wrap({
        let parser = array() | object() | number() | string() | "true" | "false" | "null";
        move |input| parser.parse(input)
      }));
      let value = || {
        let value_rc = value_rc.clone();
        wrap(move |input| value_rc.parse(input))
      };

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
