mod common;
mod lexer;
mod parser;

use criterion::{criterion_group, criterion_main, Criterion};
use lexer::lexer_entry;
use parser::{parser_entry_with_recur, parser_entry_with_static};
use std::fs::read_to_string;
use whitehole::{action::Action, combinator::Combinator, parser::Parser};

fn process(entry: Combinator<impl Action<Text = str, State = (), Heap = (), Value = ()>>, s: &str) {
  let mut parser = Parser::builder().entry(entry).build(s);

  // consume the whole input
  for _ in &mut parser {}

  let rest = parser.instant.rest();
  if !rest.is_empty() {
    panic!(
      "failed to consume the whole input, remaining: {:?}",
      &rest[..100.min(rest.len())]
    );
  }
}

fn bench_with(name: &str, parser: impl Fn(&str), c: &mut Criterion) {
  // json files are from https://github.com/miloyip/nativejson-benchmark/tree/478d5727c2a4048e835a29c65adecc7d795360d5/data
  // you may need to download them manually
  let citm_catalog = read_to_string("bench_data/citm_catalog.json").unwrap();
  let twitter = read_to_string("bench_data/twitter.json").unwrap();
  let canada = read_to_string("bench_data/canada.json").unwrap();

  let total_bytes = citm_catalog.len() + twitter.len() + canada.len();

  c.bench_function(
    &format!(
      "{}: process 3 json files (total {} bytes)",
      name, total_bytes
    ),
    |b| {
      b.iter(|| {
        parser(&citm_catalog);
        parser(&twitter);
        parser(&canada);
      })
    },
  );
}

fn lex_json(c: &mut Criterion) {
  fn lex(s: &str) {
    process(lexer_entry(), s);
  }
  bench_with("lex_json", lex, c);
}

fn parse_json_with_recur(c: &mut Criterion) {
  fn parse_with_recur(s: &str) {
    process(parser_entry_with_recur(), s);
  }
  bench_with("parse_json_with_recur", parse_with_recur, c);
}

fn parse_json_with_static(c: &mut Criterion) {
  fn parse_with_static(s: &str) {
    process(parser_entry_with_static(), s);
  }
  bench_with("parse_json_with_static", parse_with_static, c);
}

criterion_group! {
  name = benches;
  config = Criterion::default();
  targets = lex_json, parse_json_with_recur, parse_json_with_static
}
criterion_main!(benches);
