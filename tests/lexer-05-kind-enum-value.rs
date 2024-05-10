use whitehole::lexer::{
  action::regex,
  token::{token_kind, SubTokenKind},
  LexerBuilder,
};

// define token kinds, make sure it is decorated by `#[token_kind]`
#[token_kind]
#[derive(Clone, Debug)]
enum MyKind {
  A(usize),
}

#[test]
fn kind_id_is_not_relevant_with_value() {
  // kind id is bound to the sub token kind, not the value
  A::kind_id();
}

#[test]
fn kind_enum_with_calculated_value() {
  // if we want to calculate the value by the action's output
  // we need to use `action.select`
  let action = regex(r"^\d+")
    .into_action()
    .select(|ctx| A(ctx.content().parse().unwrap()));

  // these actions already have target token kind set,
  // so we can't use `define` to add them to the lexer builder,
  // we should use `append`
  let mut lexer = LexerBuilder::new().append(action).build("123");

  // the lex should emit `A(123)`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind.value(), MyKind::A(A(123))));
}

#[test]
fn kind_enum_with_const_value() {
  // if the value is a constant, we can still use `action.bind` and `builder.define`
  // then all tokens emitted by the action will have the same value
  let mut lexer = LexerBuilder::new()
    .append(regex(r"^a").into_action().bind(A(42)))
    .define(A(66), regex(r"^b"))
    .build("aabb");
  assert!(matches!(
    lexer.lex().token.unwrap().kind.value(),
    MyKind::A(A(42))
  )); // the first lex for 42
  assert!(matches!(
    lexer.lex().token.unwrap().kind.value(),
    MyKind::A(A(42))
  )); // the second lex for 42
  assert!(matches!(
    lexer.lex().token.unwrap().kind.value(),
    MyKind::A(A(66))
  )); // the first lex for 66
  assert!(matches!(
    lexer.lex().token.unwrap().kind.value(),
    MyKind::A(A(66))
  )); // the second lex for 66
}
