use std::rc::Rc;
use whitehole::lexer::{
  action::{regex, whitespaces},
  token::token_kind,
  Lexer, LexerBuilder,
};

// define token kinds, make sure it is decorated by `#[token_kind]`
#[token_kind]
#[derive(Clone, Default)]
enum MyKind {
  #[default]
  Anonymous,
  A,
}

#[test]
fn stateless_lexer() {
  // `Lexer` is stateful, it keep tracks of the lexer state (digested, text, etc) and the action state.
  // we can use `builder.build_stateless` to get a stateless lexer
  LexerBuilder::<MyKind>::default().build_stateless();

  // or use `lexer.stateless()` to get the stateless lexer from a stateful lexer
  let lexer = LexerBuilder::new()
    .ignore_default(whitespaces())
    .define(A, regex(r"^a"))
    .build(" a");
  // in this case the stateless lexer is wrapped in an `Rc`
  // so you can clone it
  let stateless = lexer.stateless().clone();

  // stateless lexer is useful if you only want to
  // lex the head of an input text, with the default action state
  let output = stateless.lex("aaa");
  assert!(matches!(output.0.token.unwrap().kind.value(), MyKind::A));

  // you can also manually provide the action state and other details
  let mut state = ();
  let output = stateless.lex_with("aaa", |o| o.start(1).state(&mut state));
  assert!(matches!(output.token.unwrap().kind.value(), MyKind::A));
}

#[test]
fn stateless_to_lexer() {
  // if you already have all actions, but don't have an input text yet,
  // you can build a stateless lexer first, and then build a stateful lexer from it
  let stateless = LexerBuilder::<MyKind>::default().build_stateless();
  let lexer = Lexer::new(Rc::new(stateless), (), "123");
  assert_eq!(lexer.state().text(), "123");
}
