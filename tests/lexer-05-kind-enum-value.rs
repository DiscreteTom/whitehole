use whitehole::lexer::{action::regex, token::TokenKind, LexerBuilder};
use whitehole_macros::TokenKind;
use MyKind::*; // use the enum variants directly

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone, Debug)]
enum MyKind {
  A(usize),
}

// for convenience, we can implement `From<T>` for the kind
impl From<usize> for MyKind {
  fn from(value: usize) -> Self {
    A(value)
  }
}

#[test]
fn kind_id_is_not_relevant_with_value() {
  // we can have enum variants with values as the token kinds
  // the kind id is not related to the value
  assert_eq!(A(0).id(), A(1).id());
}

#[test]
fn kind_enum_with_calculated_value() {
  // if we want to calculate the value by the action's output
  // we need to use `action.kinds` and `action.select`
  let action = regex(r"^a")
    .unwrap()
    // in `kinds` the value is not important, we just want to get the kind id
    .kinds([A(Default::default())])
    .select(|ctx| A(ctx.output.rest().len()));

  // yes we can use `append` and `append_with` to use an action with possible_kinds set
  let mut lexer = LexerBuilder::<MyKind>::default().append(action).build("aa");

  // the first lex should be accepted as `A(1)`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, A(1)));

  // the second lex should be accepted as `A(0)`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, A(0)));

  // be ware, when lex with expectation, only the kind id is compared
  // the value is ignored. in the following example
  // even we expect `A(0)`, the lex will still accept `A(1)`
  let mut lexer = lexer.clone_with_default_action_state("aa");
  let token = lexer.lex_expect(&A(0)).token.unwrap();
  assert!(matches!(token.kind, A(1)));
}

#[test]
fn kind_enum_with_const_value() {
  // if the value is a constant, we can still use `action.bind` and `builder.define`
  // then all token yielded by the action will have the same value
  let mut lexer = LexerBuilder::<MyKind>::default()
    .append(regex(r"^a").unwrap().bind(A(42)))
    .define(A(66), regex(r"^b").unwrap())
    .build("aabb");
  assert!(matches!(lexer.lex().token.unwrap().kind, A(42))); // the first lex for 42
  assert!(matches!(lexer.lex().token.unwrap().kind, A(42))); // the second lex for 42
  assert!(matches!(lexer.lex().token.unwrap().kind, A(66))); // the first lex for 66
  assert!(matches!(lexer.lex().token.unwrap().kind, A(66))); // the second lex for 66
}

#[test]
fn into_kind_enum() {
  // `action.bind` accept `impl Into<YourKind>` as the parameter
  // so if YourKind implements `From<T>` you can use `T` directly
  assert!(matches!(
    LexerBuilder::<MyKind>::default()
      .append(
        regex(r"^a")
          .unwrap()
          .bind(42) // here, use `42` directly
      )
      .build("aa")
      .lex()
      .token
      .unwrap()
      .kind,
    A(42)
  ));

  // `builder.define` and `builder.define_with` also accept `impl Into<YourKind>`
  assert!(matches!(
    LexerBuilder::<MyKind>::default()
      // here, use `42` directly
      .define(42, regex(r"^a").unwrap())
      .build("aa")
      .lex()
      .token
      .unwrap()
      .kind,
    A(42)
  ));
  assert!(matches!(
    LexerBuilder::<MyKind>::default()
      // here, use `42` directly
      .define(42, regex(r"^a").unwrap())
      .build("aa")
      .lex()
      .token
      .unwrap()
      .kind,
    A(42)
  ));
}
