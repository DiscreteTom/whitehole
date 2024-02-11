use whitehole::lexer::{
  expectation::Expectation, stateless::lex::StatelessLexOptions, Action, Builder, Lexer,
};
use whitehole_macros::TokenKind;

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
  Builder::<MyKind, (), ()>::default().build_stateless();

  // or use `lexer.stateless()` to get the stateless lexer from a stateful lexer
  let lexer = Builder::<MyKind, (), ()>::default()
    .ignore(Action::regex(r"^\s+").unwrap().bind(MyKind::Anonymous))
    .define(MyKind::A, Action::regex(r"^a").unwrap())
    .build(" a");
  // in this case the stateless lexer is wrapped in a `Rc`
  // so we can clone it
  let stateless = lexer.stateless().clone();

  // stateless lexer is useful if we only want to
  // lex the head of a input buffer, with the default action state
  let output = stateless.lex("aaa");
  assert!(matches!(output.token.unwrap().kind, MyKind::A));

  // we can also manually provide the action state and other details
  let output = stateless.lex_with(
    "aaa",
    StatelessLexOptions {
      action_state: &mut (),
      start: 0,
      expectation: Expectation::default(),
    },
  );
  assert!(matches!(output.token.unwrap().kind, MyKind::A));
}

#[test]
fn stateless_to_lexer() {
  // if we already have all actions, but we don't have a buffer
  // we can build a stateless lexer first, and then build a stateful lexer from it
  let stateless = Builder::<MyKind, (), ()>::default().build_stateless();

  // this will consume the stateless lexer
  let lexer = stateless.into_lexer("123");
  assert_eq!(lexer.state().buffer(), "123");

  // if we have a Rc<StatelessLexer> instead of a raw StatelessLexer
  // e.g. we get it from a stateful lexer
  let stateless = lexer.stateless().clone();
  // we can just use the `Lexer::new` to create a stateful lexer
  let lexer = Lexer::new(stateless, "123");
  assert_eq!(lexer.state().buffer(), "123");
}
