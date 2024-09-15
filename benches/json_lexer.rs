use criterion::{criterion_group, criterion_main, Criterion};
use std::{fs::read_to_string, rc::Rc};
use whitehole::lexer::{
  action::{
    chars, exact,
    json::{boundaries, number_with, string_with},
    FloatLiteralData, HexEscapeError, PartialStringBody,
  },
  builder::LexerBuilder,
  into::IntoLexer,
  stateless::StatelessLexer,
  token::token_kind,
};

#[token_kind]
#[derive(Default, Clone, Debug)]
enum JsonTokenKind {
  #[default]
  Anonymous,
  JsonString(Vec<PartialStringBody<String, HexEscapeError>>),
  Number(FloatLiteralData<Vec<usize>, String, String, String>),
  True,
  False,
  Null,
}

fn build_lexer() -> StatelessLexer<'static, JsonTokenKind> {
  LexerBuilder::new()
    .ignore_default(chars(|c| matches!(c, ' ' | '\n' | '\r' | '\t')))
    .append_default(boundaries())
    .define(True, exact("true"))
    .define(False, exact("false"))
    .define(Null, exact("null"))
    .append(
      string_with(|o| o.acc(Vec::new())).select(|ctx| JsonString(ctx.output.binding.take().data)),
    )
    .append(
      number_with(|o| {
        o.separator(Vec::new())
          .integer(String::new())
          .fraction(String::new())
          .exponent(String::new())
      })
      .select(|ctx| Number(ctx.output.binding.take().data)),
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
  c.bench_function("json_lexer: build", |b| b.iter(build_lexer));
}

fn bench_lex(c: &mut Criterion) {
  // json files are from https://github.com/miloyip/nativejson-benchmark/
  // you may need to download them manually
  let citm_catalog = read_to_string("bench_data/citm_catalog.json").unwrap();
  let twitter = read_to_string("bench_data/twitter.json").unwrap();
  let canada = read_to_string("bench_data/canada.json").unwrap();

  let stateless = Rc::new(build_lexer());

  c.bench_function("json_lexer: lex 3 json", |b| {
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
