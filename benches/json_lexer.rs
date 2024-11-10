use criterion::{criterion_group, criterion_main, Criterion};
use std::fs::read_to_string;
use whitehole::{
  combinator::{chars, exact, next, Combinator},
  parser::Builder,
};

fn build_lexer() -> Builder<Combinator<'static, ()>, (), ()> {
  let whitespaces = chars(|c| " \t\r\n".contains(c));
  let number = {
    let digit_1_to_9 = next(|c| ('1'..='9').contains(&c));
    let digits = || chars(|c| c.is_ascii_digit());
    let integer = exact('0') | (digit_1_to_9 + digits().accept());
    let fraction = exact('.') + digits();
    let exponent = (exact('e') | 'E') + (exact('-') | '+').accept() + digits();
    exact('-').accept() + integer + fraction.accept() + exponent.accept()
  };
  let string = {
    let escape = exact('\\')
      + (next(|c| "\"\\/bfnrt".contains(c)) | (exact('u') + next(|c| c.is_ascii_hexdigit()) * 4));
    let non_escape = chars(|c| c != '"' && c != '\\' && ('\u{0020}'..='\u{10ffff}').contains(&c));
    let body = (escape | non_escape) * ..;
    exact('"') + body.accept() + exact('"')
  };
  let boundary = next(|c| "[]{}:,".contains(c));

  Builder::new().entry(whitespaces | boundary | number | string | "true" | "false" | "null")
}

fn lex_json(builder: Builder<Combinator<'static, ()>, (), ()>, s: &str) {
  let mut parser = builder.build(s);

  loop {
    let output = parser.parse();
    if output.is_none() {
      break;
    }
    // println!("{:?}", output);
  }

  if !parser.rest().is_empty() {
    panic!(
      "lexer failed to consume the whole input, remaining: {}",
      &parser.rest()[..100.min(parser.rest().len())]
    );
  }
}

fn bench_lex(c: &mut Criterion) {
  // json files are from https://github.com/miloyip/nativejson-benchmark/
  // you may need to download them manually
  let citm_catalog = read_to_string("bench_data/citm_catalog.json").unwrap();
  let twitter = read_to_string("bench_data/twitter.json").unwrap();
  let canada = read_to_string("bench_data/canada.json").unwrap();

  c.bench_function("json_lexer: lex 3 json", |b| {
    b.iter(|| {
      lex_json(build_lexer(), &citm_catalog);
      lex_json(build_lexer(), &twitter);
      lex_json(build_lexer(), &canada);
    })
  });
}

criterion_group! {
  name = benches;
  config = Criterion::default();
  targets = bench_lex
}
criterion_main!(benches);
