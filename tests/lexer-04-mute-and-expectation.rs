use whitehole::lexer::{
  action::{exact, regex, whitespaces, Action},
  token::{token_kind, SubTokenKind},
  LexerBuilder,
};

// define token kinds, make sure it is decorated by `#[token_kind]`
#[token_kind]
#[derive(Clone, Default)]
enum MyKind {
  #[default]
  Anonymous,
  A,
  B,
}

#[test]
fn muted_actions() {
  // there is a field `muted` in `Action`
  let action: Action<_> = regex(r"^a");
  // by default `muted` is `false`
  assert!(!action.muted());
  // with `mute`, the `muted` field will be set to `true`
  let action = action.mute();
  assert!(action.muted());
}

#[test]
fn expectation() {
  let mut lexer = LexerBuilder::new()
    // with `ignore`, the `LexerBuilder` will make actions muted
    .ignore_default(whitespaces())
    .define(A, exact("a"))
    .define(B, exact("a"))
    .define(B, exact("b"))
    .build("\ta");

  // by default, the lex will evaluate all actions in the order they are defined
  // so the lex should be accepted as `A`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.binding.kind(), MyKind::A));

  // but if we have an expected kind
  // the lex will only evaluate actions
  // which are bound to the expected kind or muted
  let mut lexer = lexer.reload("\ta");
  let res = lexer.lex_with(|o| o.expect(B::kind_id()));
  let token = res.token.unwrap();
  assert!(matches!(token.binding.kind(), MyKind::B));
  assert_eq!(res.digested, 2); // the muted action was also evaluated and digested a byte

  // we can also expect a literal if the literal appears in `exact` or `word`
  let mut lexer = lexer.reload("\ta");
  let res = lexer.lex_with(|o| o.expect("b"));
  let token = res.token;
  assert!(token.is_none());
  assert_eq!(res.digested, 1); // the muted action is also evaluated and digested a byte
  let token = lexer.lex_with(|o| o.expect("a")).token.unwrap();
  assert!(matches!(token.binding.kind(), MyKind::A));

  // or both the text and the kind are expected
  let mut lexer = lexer.reload("\ta");
  assert!(lexer
    .lex_with(|o| o.expect_with(|e| e.literal("b").kind(A)))
    .token
    .is_none());
  assert!(lexer
    .lex_with(|o| o.expect_with(|e| e.literal("a").kind(A)))
    .token
    .is_some());
}
