use whitehole::lexer::{token::TokenKind, Action, Builder};
use whitehole_macros::TokenKind;

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone)]
enum MyKind {
  A(usize),
}

// for convenience, we can implement `From<T>` for the kind
impl From<usize> for MyKind {
  fn from(value: usize) -> Self {
    MyKind::A(value)
  }
}

#[test]
fn kind_enum_value() {
  // we can have enum variants with values as the token kinds
  // the kind id is not related to the value
  assert_eq!(MyKind::A(0).id(), MyKind::A(1).id());

  // if the value is a constant, we can still use `action.bind`
  // then all token yielded by the action will have the same value
  let mut lexer = Builder::<MyKind, (), ()>::default()
    .append(
      Action::<(), (), ()>::regex(r"^a")
        .unwrap()
        .bind::<MyKind>(MyKind::A(42)),
    )
    .build("aa");
  assert!(matches!(lexer.lex().token.unwrap().kind, MyKind::A(42))); // the first lex
  assert!(matches!(lexer.lex().token.unwrap().kind, MyKind::A(42))); // the second lex

  // `action.bind` accept `impl Into<YourKind>` as the parameter
  // so if YourKind implements `From<T>` you can use `T` directly
  assert!(matches!(
    Builder::<MyKind, (), ()>::default()
      .append(
        Action::<(), (), ()>::regex(r"^a")
          .unwrap()
          .bind::<MyKind>(42) // here, use `42` directly
      )
      .build("aa")
      .lex()
      .token
      .unwrap()
      .kind,
    MyKind::A(42)
  ));

  // `builder.define` and `builder.define_from` also accept `impl Into<YourKind>`
  assert!(matches!(
    Builder::<MyKind, (), ()>::default()
      // here, use `42` directly
      .define(42, Action::regex(r"^a").unwrap())
      .build("aa")
      .lex()
      .token
      .unwrap()
      .kind,
    MyKind::A(42)
  ));
  assert!(matches!(
    Builder::<MyKind, (), ()>::default()
      // here, use `42` directly
      .define_from(42, |a| a.regex(r"^a").unwrap())
      .build("aa")
      .lex()
      .token
      .unwrap()
      .kind,
    MyKind::A(42)
  ));

  // if we want to calculate the value by the action's output
  // we need to use `action.kinds` and `action.select`
  let action = Action::<(), (), ()>::regex(r"^a")
    .unwrap()
    // in `kinds` the value is not important, we just want to get the kind id
    .kinds(&[&MyKind::A(0)])
    .select(|ctx| MyKind::A(ctx.output.rest().len()));

  // yes we can use `append` and `append_from` to use an action with possible_kinds set
  let mut lexer = Builder::<MyKind, (), ()>::default()
    .append(action)
    .build("aa");

  // the first lex should be accepted as `MyKind::A(1)`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, MyKind::A(1)));

  // the second lex should be accepted as `MyKind::A(0)`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, MyKind::A(0)));

  // be ware, when lex with expectation, only the kind id is compared
  // the value is ignored. in the following example
  // even we expect `MyKind::A(0)`, the lex will still accept `MyKind::A(1)`
  let mut lexer = lexer.clone_with("aa");
  let token = lexer.lex_expect(&MyKind::A(0)).token.unwrap();
  assert!(matches!(token.kind, MyKind::A(1)));
}
