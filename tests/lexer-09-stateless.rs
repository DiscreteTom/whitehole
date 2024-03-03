use whitehole::lexer::{action::regex, stateless::StatelessLexOptions, Lexer, LexerBuilder};
use whitehole_macros::TokenKind;
use MyKind::*; // use the enum variants directly

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone)]
enum MyKind {
  Anonymous,
  A,
}

#[test]
fn stateless_lexer() {
  // `Lexer` is stateful, it keep tracks of the lexer state (digested, buffer, etc) and the action state.
  // we can use `builder.build_stateless` to get a stateless lexer
  LexerBuilder::<MyKind>::default().build_stateless();

  // or use `lexer.stateless()` to get the stateless lexer from a stateful lexer
  let lexer = LexerBuilder::<MyKind>::default()
    .ignore(regex(r"^\s+").unwrap().bind(Anonymous))
    .define(A, regex(r"^a").unwrap())
    .build(" a");
  // in this case the stateless lexer is wrapped in a `Rc`
  // so we can clone it
  let stateless = lexer.stateless().clone();

  // stateless lexer is useful if we only want to
  // lex the head of a input buffer, with the default action state
  let output = stateless.lex("aaa");
  assert!(matches!(output.token.unwrap().kind, A));

  // we can also manually provide the action state and other details
  let output = stateless.lex_with("aaa", StatelessLexOptions::with_action_state(&mut ()));
  assert!(matches!(output.token.unwrap().kind, A));
}

#[test]
fn stateless_to_lexer() {
  // if we already have all actions, but we don't have a buffer
  // we can build a stateless lexer first, and then build a stateful lexer from it
  let stateless = LexerBuilder::<MyKind>::default().build_stateless();

  // this will consume the stateless lexer
  let lexer = stateless.into_lexer("123");
  assert_eq!(lexer.state().text(), "123");

  // if we have a Rc<StatelessLexer> instead of a raw StatelessLexer
  // e.g. we get it from a stateful lexer
  let stateless = lexer.stateless().clone();
  // we can just use the `Lexer::new` to create a stateful lexer
  let lexer = Lexer::with_default_action_state(stateless, "123");
  assert_eq!(lexer.state().text(), "123");
}
