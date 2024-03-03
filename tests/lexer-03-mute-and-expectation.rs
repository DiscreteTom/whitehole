use whitehole::lexer::{
  action::regex, expectation::Expectation, token::TokenKind, Action, LexerBuilder,
};
use whitehole_macros::TokenKind;
use MyKind::*; // use the enum variants directly

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone, Default)]
enum MyKind {
  // as a convention, we use `Anonymous` as the default kind
  // so that we can use `builder.ignore_default`
  #[default]
  Anonymous,
  A,
  B,
}

#[test]
fn maybe_muted() {
  // there is a field `maybe_muted` in `Action`
  // which will be set if you use `action.mute` or `action.mute_if`
  let action: Action<()> = regex(r"^a").unwrap();
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
  let mut action: Action<()> = regex(r"^a").unwrap();
  action.maybe_muted = true;
  assert!(action.maybe_muted);
}

#[test]
fn builder_ignore() {
  // when you use `builder.ignore` or `builder.ignore_with`
  // the builder will set the `maybe_muted` field to `true`
  assert!(
    LexerBuilder::<MyKind>::default()
      .ignore(regex("^-").unwrap().bind(Anonymous))
      .build_stateless()
      .actions()[0]
      .maybe_muted
  );
  assert!(
    LexerBuilder::<MyKind>::default()
      .ignore_with(|a| a.regex("^-").unwrap().bind(Anonymous).into())
      .build_stateless()
      .actions()[0]
      .maybe_muted
  );

  // if your token kind implements `Default` and `Clone`
  // you can use `builder.ignore_default` or `builder.ignore_default_with`
  // so that the builder will bind the A with the default kind
  let stateless = LexerBuilder::<MyKind>::default()
    .ignore_default(regex("^-").unwrap())
    .build_stateless();
  let action = &stateless.actions()[0];
  assert!(action.maybe_muted);
  assert_eq!(action.possible_kinds().len(), 1);
  assert!(action.possible_kinds().contains(&Anonymous.id()));
  let stateless = LexerBuilder::<MyKind>::default()
    .ignore_default_with(|a| a.regex("^-").unwrap().into())
    .build_stateless();
  let action = &stateless.actions()[0];
  assert!(action.maybe_muted);
  assert_eq!(action.possible_kinds().len(), 1);
  assert!(action.possible_kinds().contains(&Anonymous.id()));
}

#[test]
fn expectation() {
  let mut lexer = LexerBuilder::<MyKind>::default()
    .ignore(regex("^-").unwrap().bind(Anonymous))
    .define(A, regex(r"a").unwrap())
    .define(B, regex(r"a").unwrap())
    .build("-a");

  // by default, the lex will evaluate all actions in the order they are defined
  // so the first lex should be accepted as `A`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, A));

  // but if we have an expected kind
  // the lex will only evaluate actions which are bound to the expected kind
  // or maybe-muted actions
  let mut lexer = lexer.reload("-a");
  let res = lexer.lex_expect(&B);
  let token = res.token.unwrap();
  assert!(matches!(token.kind, B));
  assert_eq!(res.digested, 2); // the muted action is also evaluated and digested a character

  // we can also expect a specific text
  let mut lexer = lexer.reload("-a");
  let res = lexer.lex_expect("b");
  let token = res.token;
  assert!(token.is_none());
  assert_eq!(res.digested, 1); // the muted action is also evaluated and digested a character
  let token = lexer.lex_expect("a").token.unwrap();
  assert!(matches!(token.kind, A));

  // or both the text and the kind are expected
  let mut lexer = lexer.reload("-a");
  assert!(lexer
    .lex_expect(Expectation::from("b").kind(A))
    .token
    .is_none());
  assert!(lexer
    .lex_expect(Expectation::from("a").kind(A))
    .token
    .is_some());
}
