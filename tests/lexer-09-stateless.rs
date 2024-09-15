use std::rc::Rc;
use whitehole::{
  kind::whitehole_kind,
  lexer::{
    action::{regex, whitespaces},
    builder::LexerBuilder,
    Lexer,
  },
};

// define token kinds, make sure it is decorated by `#[whitehole_kind]`
#[whitehole_kind]
#[derive(Clone, Default)]
enum MyKind {
  #[default]
  Anonymous,
  A,
}

#[test]
fn stateless_lexer() {
  // `Lexer` is stateful, it keep tracks of the lexer state (digested, text, etc) and the state.
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
  // lex the head of an input text, with the default state
  let output = stateless.lex("aaa");
  assert!(matches!(output.0.token.unwrap().binding.kind(), MyKind::A));

  // you can also manually provide the state and other details
  let mut state = ();
  let mut heap = ();
  let output = stateless.lex_with("aaa", |o| o.start(1).state(&mut state).heap(&mut heap));
  assert!(matches!(output.token.unwrap().binding.kind(), MyKind::A));
}

#[test]
fn stateless_to_lexer() {
  // if you already have all actions, but don't have an input text yet,
  // you can build a stateless lexer first, and then build a stateful lexer from it
  let stateless = LexerBuilder::<MyKind>::default().build_stateless();
  let lexer = Lexer::new(Rc::new(stateless), (), (), "123");
  assert_eq!(lexer.instant().text(), "123");
}
