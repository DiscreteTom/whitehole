use regex::Regex;
use whitehole::lexer::{
  action::{regex, simple_with_data, Action},
  token::{token_kind, MockTokenKind, SubTokenKind, TokenKindIdProvider},
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
  let action = regex(r"^\d+").select(|ctx| A(ctx.content().parse().unwrap()));

  // these actions already have target token kind set,
  // so we can't use `define` to add them to the lexer builder,
  // we should use `append` instead of `define`
  let mut lexer = LexerBuilder::new().append(action).build("123");

  // the lex should emit `A(123)`
  let token = lexer.lex().token.unwrap();
  // as you can see the `MyKind::A(usize)` enum is mutated to `MyKind::A(A)` by the `#[token_kind]` macro
  assert!(matches!(token.kind.value(), MyKind::A(A(123))));
}

#[test]
fn kind_enum_with_const_value() {
  // if the value is a constant, we can still use `action.bind` and `builder.define`
  // then all tokens emitted by the action will have the same value
  let mut lexer = LexerBuilder::new()
    .append(regex(r"^a").bind(A(42)))
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

#[test]
fn carry_data_with_mock_token_kind() {
  // `MockTokenKind<()>` is the default action kind
  let _: Action<MockTokenKind<()>> = regex(r"^a");

  // `MockTokenKind` can carry data, it implements `SubTokenKind` and `TokenKindIdProvider`,
  // and it will always have the same kind id
  assert_eq!(MockTokenKind::new(123).id(), MockTokenKind::kind_id());
  assert_eq!(MockTokenKind::new("123").id(), MockTokenKind::kind_id());

  // with this you can calculate data during the action is executed
  // instead of parsing the token's content in `Action::select`.
  let action: Action<MockTokenKind<usize>> = simple_with_data(|input| {
    Regex::new(r"^(\d+)e(\d+)")
      .unwrap()
      .captures(input.rest())
      .map(|m| {
        let a: usize = m[1].parse().unwrap();
        let b: u32 = m[2].parse().unwrap();
        (
          m[0].len(),             // how many bytes are digested
          (a * usize::pow(a, b)), // custom data
        )
      })
  });

  // now convert the `MockTokenKind` to your token kind with `Action::select`
  let action = action.select(|ctx| A(ctx.output.kind.data));

  // you can construct actions with `MockTokenKind` using `Action::data`
  let action: Action<MockTokenKind<usize>> = action.data(|ctx| match ctx.output.kind.take() {
    MyKind::A(a) => a.0,
  });

  // you can also transform the data with `Action::map`
  let _: Action<MockTokenKind<String>> = action.map(|data| data.to_string());

  // TODO: add examples after action utils for strings and numbers are implemented
}
