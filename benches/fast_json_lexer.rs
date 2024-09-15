use criterion::{criterion_group, criterion_main, Criterion};
use std::{fs::read_to_string, rc::Rc};
use whitehole::{
  kind::whitehole_kind,
  lexer::{
    action::{chars, simple, unchecked_exact, unchecked_exact_chars},
    builder::LexerBuilder,
    into::IntoLexer,
    stateless::StatelessLexer,
  },
};

#[whitehole_kind]
#[derive(Default, Clone, Debug)]
enum JsonTokenKind {
  #[default]
  Anonymous,
  JsonString,
  Number,
  True,
  False,
  Null,
}

fn build_lexer() -> StatelessLexer<'static, JsonTokenKind> {
  // this lexer assume the input is a valid json and won't check the correctness
  LexerBuilder::new()
    .ignore_default(chars(|c| matches!(c, ' ' | '\n' | '\r' | '\t')))
    .append_default(unchecked_exact_chars("{},:[]"))
    .define(True, unchecked_exact("true"))
    .define(False, unchecked_exact("false"))
    .define(Null, unchecked_exact("null"))
    .define(
      JsonString,
      simple(|input| {
        let mut digested = input.next().len_utf8(); // the open quote
        let mut escaped = false;
        for c in input.rest().chars().skip(1) {
          digested += c.len_utf8();
          if c == '"' && !escaped {
            break;
          }
          escaped = c == '\\';
        }
        digested
      })
      .unchecked_head_in(['"']),
    )
    .define(
      Number,
      chars(|c| !matches!(c, ',' | ']' | '}' | ' ' | '\n' | '\r' | '\t'))
        .unchecked_head_in(['-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9']),
    )
    .build_stateless()
}

fn lex_json(stateless: &Rc<StatelessLexer<JsonTokenKind>>, s: &str) {
  let mut lexer = stateless.clone().into_lexer(s);

  loop {
    let output = lexer.lex();
    if output.digested == 0 {
      break;
    }
    // println!("{:?}", output);
  }

  if !lexer.instant().rest().is_empty() {
    panic!("lexer failed to consume the whole input");
  }
}

fn bench_build(c: &mut Criterion) {
  c.bench_function("fast_json_lexer: build", |b| b.iter(build_lexer));
}

fn bench_lex(c: &mut Criterion) {
  // json files are from https://github.com/miloyip/nativejson-benchmark/
  // you may need to download them manually
  let citm_catalog = read_to_string("bench_data/citm_catalog.json").unwrap();
  let twitter = read_to_string("bench_data/twitter.json").unwrap();
  let canada = read_to_string("bench_data/canada.json").unwrap();

  let stateless = Rc::new(build_lexer());

  c.bench_function("fast_json_lexer: lex 3 json", |b| {
    b.iter(|| {
      lex_json(&stateless, &citm_catalog);
      lex_json(&stateless, &twitter);
      lex_json(&stateless, &canada);
    })
  });
}

criterion_group! {
  name = benches;
  config = Criterion::default();
  targets = bench_build, bench_lex
}
criterion_main!(benches);
