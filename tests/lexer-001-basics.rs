use whitehole::lexer::{action::output::ActionOutputWithoutKind, Action, Builder};
use whitehole_macros::TokenKind;

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone)]
enum MyKind {
  Anonymous,
  A,
  B,
  C,
}

#[test]
fn lexer_basic() {
  // create a lexer via the lexer builder
  // specify lexer builder's generic parameters explicitly
  // the first parameter is the token kind
  // the second parameter is the action state (not used in this example)
  // the third parameter is the error type (not used in this example)
  let mut lexer = Builder::<MyKind, (), ()>::default()
    // you can use `ignore` to define a muted action
    // which will be accepted during the lexing process
    // without yielding any token
    .ignore(
      // we can create actions from a regex pattern
      // and use `bind` to bind the action to a token kind
      // remember to use `^` to match the start of the rest string
      Action::regex(r"^\s+").unwrap().bind(MyKind::Anonymous),
    )
    // for not muted actions, we can use `define` to define them
    // the first parameter is the target token kind
    // the second parameter is the action
    .define(
      MyKind::A,
      // when using `Action::simple`
      // the closure's return value indicates how many characters are digested by the action
      // `0` means the action is rejected
      // we don't need to call `bind` here because the action will be bound to `MyKind::A`
      Action::simple(|input| if input.rest().starts_with("a") { 1 } else { 0 }),
    )
    .define(
      MyKind::B,
      // yes we can use regex here too
      Action::regex("^b").unwrap(),
    )
    .define(
      MyKind::C,
      // if you want to control more details about the action's output
      // like the `error` field and the `muted` field
      // you can use `Action::new` to create an action
      // which will directly return an `Option<ActionOutput>`
      // however this is NOT the simplest way to modify the action
      // we will introduce a simpler way in `lexer-002-actions.rs`
      Action::new(|input| {
        if input.rest().starts_with("c") {
          Some(ActionOutputWithoutKind {
            digested: 1,
            error: None,
            muted: false,
          })
        } else {
          None
        }
      }),
    )
    // load the input string
    .build("a b c");

  // the first token should be `MyKind::A`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind(), MyKind::A));
  assert_eq!(token.start(), 0);
  assert_eq!(token.end(), 1);
  assert_eq!(token.content(), "a");
  assert!(matches!(token.error(), None));

  // the second token should be `MyKind::B`
  // because whitespace is muted and ignored
  // no token will be yielded for it
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind(), MyKind::B));
  assert_eq!(token.start(), 2);
  assert_eq!(token.end(), 3);
  assert_eq!(token.content(), "b");
  assert!(matches!(token.error(), None));

  // the third token should be `MyKind::C`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind(), MyKind::C));
  assert_eq!(token.start(), 4);
  assert_eq!(token.end(), 5);
  assert_eq!(token.content(), "c");
  assert!(matches!(token.error(), None));
}
