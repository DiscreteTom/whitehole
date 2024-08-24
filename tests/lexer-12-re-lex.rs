use whitehole::lexer::{
  action::{exact, whitespaces},
  token::token_kind,
  LexerBuilder,
};

// define token kinds, make sure it is decorated by `#[token_kind]`
#[token_kind]
#[derive(Clone, Default)]
enum MyKind {
  #[default]
  Anonymous,
  ShiftRight,
  RightAngle,
}

#[test]
fn re_lex() {
  // sometimes you want to re-lex the input
  // e.g. in rust we use `<>` for both generics and bit shift,
  // so when lexing `Option<Option<()>>`
  // the lexer may treat the `>>` as `ShiftRight` instead of two `RightAngle`.
  // this could be solved with expectational lexing when working with a parser
  // with grammar rules defined.
  // this can also be solved by re-lexing the lexer.

  let text = "Option<Option<()>>";
  let mut lexer = LexerBuilder::new()
    .ignore_default([whitespaces(), exact("Option"), exact("()"), exact("<")])
    .append([
      // as a best practice you should append longer literals first,
      // e.g. here you should put `>>` before `>`
      // so that when lexing `1 >> 2` the lexer will emit `>>` instead of two `>`
      exact(">>").bind(ShiftRight),
      exact(">").bind(RightAngle),
    ])
    .build(text);

  // take a snapshot before lexing
  let snapshot = lexer.snapshot();

  // lex, but with `fork` enabled, so that the lexer will
  // check if this lex is re-lexable
  let output = lexer.lex_with(|o| o.fork());
  // the first lex will emit `>>`, which is not what we expected
  assert_eq!(&text[output.token.unwrap().range], ">>");

  // when you figure out that `>>` shouldn't be here (e.g. by a parser grammar rule)
  // you may want to re-lex the lexer, continue lexing from the last evaluated action.
  // luckily, the output contains the state before the last lex,
  // and a re-lex context to tell the lexer to skip the evaluated actions.
  let context = output.fork.unwrap(); // if this is none, then the lex is not re-lexable
  lexer.restore(snapshot); // restore the lexer to the state before the last lex

  // provide the re-lex context to the new lexer when lex.
  // we also enable `fork` so that the lexer will check if this lex is re-lexable
  let output = lexer.lex_with(|o| o.re_lex(context).fork());
  // now the lexer will emit `>`
  assert_eq!(&text[output.token.unwrap().range], ">");
  // and there is no next action to re-lex, the re_lexable should be None
  assert!(output.fork.is_none());

  // besides, it is a best practice to trim the lexer before lex with fork enabled
  // so that muted actions won't be evaluated again

  // comparing to expectational lexing, re-lex is more powerful,
  // it doesn't require a specific kind or literal to expect,
  // but it is more expensive (the lexer will be cloned),
  // so use it wisely, e.g. when expectational lexing is not available.
}
