use whitehole::lexer::{expectation::Expectation, token::TokenKind, Action, Builder};
use whitehole_macros::TokenKind;

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
fn builder_ignore() {
  // when you use `builder.ignore` or `builder.ignore_from`
  // the builder will set the `maybe_muted` field to `true`
  assert!(
    Builder::<MyKind, (), ()>::default()
      .ignore(Action::regex("^-").unwrap().bind(MyKind::Anonymous))
      .build_stateless()
      .actions()[0]
      .maybe_muted
  );
  assert!(
    Builder::<MyKind, (), ()>::default()
      .ignore_from(|a| a.regex("^-").unwrap().bind(MyKind::Anonymous))
      .build_stateless()
      .actions()[0]
      .maybe_muted
  );

  // if your token kind implements `Default` and `Clone`
  // you can use `builder.ignore_default` or `builder.ignore_default_from`
  // so that the builder will bind the A with the default kind
  let stateless = Builder::<MyKind, (), ()>::default()
    .ignore_default(Action::regex("^-").unwrap())
    .build_stateless();
  let action = &stateless.actions()[0];
  assert!(action.maybe_muted);
  assert_eq!(action.possible_kinds().len(), 1);
  assert!(action.possible_kinds().contains(&MyKind::Anonymous.id()));
  let stateless = Builder::<MyKind, (), ()>::default()
    .ignore_default_from(|a| a.regex("^-").unwrap())
    .build_stateless();
  let action = &stateless.actions()[0];
  assert!(action.maybe_muted);
  assert_eq!(action.possible_kinds().len(), 1);
  assert!(action.possible_kinds().contains(&MyKind::Anonymous.id()));
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
  assert!(matches!(token.kind, MyKind::A));

  // but if we have an expected kind
  // the lex will only evaluate actions which are bound to the expected kind
  // or maybe-muted actions
  let mut lexer = lexer.reload("-a");
  let res = lexer.lex_expect(&MyKind::B);
  let token = res.token.unwrap();
  assert!(matches!(token.kind, MyKind::B));
  assert_eq!(res.digested, 2); // the muted action is also evaluated and digested a character

  // we can also expect a specific text
  let mut lexer = lexer.reload("-a");
  let res = lexer.lex_expect("b");
  let token = res.token;
  assert!(token.is_none());
  assert_eq!(res.digested, 1); // the muted action is also evaluated and digested a character
  let token = lexer.lex_expect("a").token.unwrap();
  assert!(matches!(token.kind, MyKind::A));

  // or both the text and the kind are expected
  let mut lexer = lexer.reload("-a");
  assert!(lexer
    .lex_expect(Expectation::from("b").kind(MyKind::A))
    .token
    .is_none());
  assert!(lexer
    .lex_expect(Expectation::from("a").kind(MyKind::A))
    .token
    .is_some());
}
