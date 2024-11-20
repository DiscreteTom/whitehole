use criterion::{criterion_group, criterion_main, Criterion};
use in_str::in_str;
use std::fs::read_to_string;
use whitehole::{
  combinator::{eat, next},
  parse::Parse,
  parser::{Builder, Parser},
};

fn build_lexer(s: &str) -> Parser<impl Parse> {
  Builder::new()
    .entry(|b| {
      let whitespaces = b.next(in_str!(" \t\r\n")) * (1..);
      let number = {
        let digit_1_to_9 = next(|c| matches!(c, '1'..='9'));
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
        let body = (escape | non_escape) * ..;
        eat('"') + body + '"'
      };
      let boundary = next(in_str!("[]{}:,"));
      whitespaces | boundary | number | string | "true" | "false" | "null"
    })
    .build(s)
}

fn lex_json(s: &str) {
  let mut parser = build_lexer(s);

  loop {
    let output = parser.parse();
    if output.is_none() {
      break;
    }
    // println!("{:?}", output);
  }

  if !parser.instant().rest().is_empty() {
    panic!(
      "lexer failed to consume the whole input, remaining: {}",
      &parser.instant().rest()[..100.min(parser.instant().rest().len())]
    );
  }
}

fn bench_lex(c: &mut Criterion) {
  // json files are from https://github.com/miloyip/nativejson-benchmark/tree/478d5727c2a4048e835a29c65adecc7d795360d5/data
  // you may need to download them manually
  let citm_catalog = read_to_string("bench_data/citm_catalog.json").unwrap();
  let twitter = read_to_string("bench_data/twitter.json").unwrap();
  let canada = read_to_string("bench_data/canada.json").unwrap();

  let total_bytes = citm_catalog.len() + twitter.len() + canada.len();

  c.bench_function(
    &format!("json_lexer: lex 3 json files (total {} bytes)", total_bytes),
    |b| {
      b.iter(|| {
        lex_json(&citm_catalog);
        lex_json(&twitter);
        lex_json(&canada);
      })
    },
  );
}

criterion_group! {
  name = benches;
  config = Criterion::default();
  targets = bench_lex
}
criterion_main!(benches);
