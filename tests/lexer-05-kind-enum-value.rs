use regex::Regex;
use whitehole::{
  kind::{kind, KindIdBinding, MockKind, SubKind},
  lexer::{
    action::{regex, simple_with_data, Action},
    builder::LexerBuilder,
  },
};

// define token kinds, make sure it is decorated by `#[kind]`
#[kind]
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
  // as you can see the `MyKind::A(usize)` enum is mutated to `MyKind::A(A)` by the `#[kind]` macro
  assert!(matches!(token.binding.kind(), MyKind::A(A(123))));
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
    lexer.lex().token.unwrap().binding.kind(),
    MyKind::A(A(42))
  )); // the first lex for 42
  assert!(matches!(
    lexer.lex().token.unwrap().binding.kind(),
    MyKind::A(A(42))
  )); // the second lex for 42
  assert!(matches!(
    lexer.lex().token.unwrap().binding.kind(),
    MyKind::A(A(66))
  )); // the first lex for 66
  assert!(matches!(
    lexer.lex().token.unwrap().binding.kind(),
    MyKind::A(A(66))
  )); // the second lex for 66
}

#[test]
fn carry_data_with_mock_kind() {
  // `MockKind<()>` is the default action kind
  let _: Action<MockKind<()>> = regex(r"^a");

  // `MockKind` can carry data, it implements `SubKind`,
  // and it will always have the same kind id
  let v1: KindIdBinding<MockKind<i32>> = MockKind::new(42).into();
  let v2: KindIdBinding<MockKind<bool>> = MockKind::new(true).into();
  assert_eq!(v1.id(), MockKind::kind_id());
  assert_eq!(v2.id(), MockKind::kind_id());

  // with this you can calculate data during the action is executed
  // instead of parsing the token's content in `Action::select`.
  let action: Action<MockKind<usize>> = simple_with_data(|input| {
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

  // now convert the `MockKind` to your token kind with `Action::select`
  let action = action.select(|ctx| A(ctx.output.binding.take().data));

  // you can construct actions with `MockKind` using `Action::data`
  let action: Action<MockKind<usize>> = action.data(|ctx| match ctx.output.binding.take() {
    MyKind::A(a) => a.0,
  });

  // you can also transform the data with `Action::map`
  let _: Action<MockKind<String>> = action.map(|data| data.to_string());

  // TODO: add examples after action utils for strings and numbers are implemented
}
