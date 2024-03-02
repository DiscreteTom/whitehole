use whitehole::lexer::{
  action::{output::ActionOutputWithoutKind, regex, simple},
  Action, LexerBuilder,
};
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

// use the enum variants directly
// e.g. we can use `A` instead of `MyKind::A`
use MyKind::*;

#[test]
fn lexer_basics() {
  // create a lexer via the lexer builder
  // specify lexer builder's generic parameters explicitly
  // the first parameter is the token kind
  // the second parameter is the action state (not used in this example)
  // the third parameter is the error type (not used in this example)
  let mut lexer = LexerBuilder::<MyKind>::default()
    // you can use `ignore` to define a muted action
    // which will be accepted during the lexing process
    // without yielding any token
    .ignore(
      // we can create actions from a regex pattern
      // and use `bind` to bind the action to a token kind
      // remember to use `^` to match the start of the rest string
      regex(r"^\s+").unwrap().bind(Anonymous),
    )
    // for not muted actions, we can use `define` to define them
    // the first parameter is the target token kind
    // the second parameter is the action
    .define(
      A,
      // when using `simple`
      // the closure's return value indicates how many characters are digested by the action
      // `0` means the action is rejected
      // we don't need to call `bind` here because the action will be bound to `A`
      simple(|input| if input.rest().starts_with("a") { 1 } else { 0 }),
    )
    .define(
      B,
      // yes we can use regex here too
      regex("^b").unwrap(),
    )
    .define(
      C,
      // if you want to control more details about the action's output
      // like the `error` field and the `muted` field
      // you can use `Action::new` to create an action
      // the closure should directly return an `Option<ActionOutput>`
      // however this is NOT the simplest way to modify the action
      // we will introduce a simpler way in the next chapter
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

  // the first token should be `A`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, A));
  assert_eq!(token.range.start, 0);
  assert_eq!(token.range.end, 1);
  assert_eq!(token.content, "a");
  assert!(matches!(token.error, None));

  // the second token should be `B`
  // because whitespace is muted and ignored
  // no token will be yielded for it
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, B));
  assert_eq!(token.range.start, 2);
  assert_eq!(token.range.end, 3);
  assert_eq!(token.content, "b");
  assert!(matches!(token.error, None));

  // the third token should be `C`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, C));
  assert_eq!(token.range.start, 4);
  assert_eq!(token.range.end, 5);
  assert_eq!(token.content, "c");
  assert!(matches!(token.error, None));
}
