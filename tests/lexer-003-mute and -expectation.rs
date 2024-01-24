use whitehole::lexer::{core::lex::expectation::Expectation, Action, Builder};
use whitehole_macros::TokenKind;

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone)]
enum MyKind {
  Anonymous,
  A,
  B,
}

#[test]
fn maybe_muted() {
  // there is a field `maybe_muted` in `Action`
  // which will be set if you use `action.mute` or `action.mute_if`
  let action = Action::<(), (), ()>::regex(r"^a").unwrap();
  // by default `maybe_muted` is `false`
  assert!(!action.maybe_muted);
  // with `mute`, the `maybe_muted` field will be set by the argument
  let action = action.mute(true);
  assert!(action.maybe_muted);
  let action = action.mute(false);
  assert!(!action.maybe_muted);

  // with `mute_if`, the `maybe_muted` field will always be set to `true`
  let action = action.mute_if(|_| true);
  assert!(action.maybe_muted);
  let action = action.mute_if(|_| false);
  assert!(action.maybe_muted);

  // we can edit the `maybe_muted` field if we know what we are doing
  let mut action = Action::<(), (), ()>::regex(r"^a").unwrap();
  action.maybe_muted = true;
  assert!(action.maybe_muted);
}

#[test]
fn expectation() {
  let mut lexer = Builder::<MyKind, (), ()>::default()
    .ignore(Action::regex("^-").unwrap().bind(MyKind::Anonymous))
    .define(MyKind::A, Action::regex(r"a").unwrap())
    .define(MyKind::B, Action::regex(r"a").unwrap())
    .build("-a");

  // by default, the lex will evaluate all actions in the order they are defined
  // so the first lex should be accepted as `MyKind::A`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind(), MyKind::A));

  // but if we have an expected kind
  // the lex will only evaluate actions which are bound to the expected kind
  // or maybe-muted actions
  let mut lexer = lexer.dry_clone("-a");
  let token = lexer.lex_expect(MyKind::B).token.unwrap();
  assert!(matches!(token.kind(), MyKind::B));

  // we can also expect a specific text
  let mut lexer = lexer.dry_clone("-a");
  let token = lexer.lex_expect("b").token;
  assert!(token.is_none());
  let token = lexer.lex_expect("a").token.unwrap();
  assert!(matches!(token.kind(), MyKind::A));

  // or both the text and the kind are expected
  let mut lexer = lexer.dry_clone("-a");
  assert!(lexer.lex_expect("b").kind(MyKind::A))
    .token
    .is_none();
  assert!(lexer
    .lex_expect(Expectation::from("a").kind(MyKind::A))
    .token
    .is_some());
}
