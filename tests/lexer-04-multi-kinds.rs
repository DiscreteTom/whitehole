use whitehole::lexer::{token::TokenKind, Action, Builder};
use whitehole_macros::TokenKind;

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone)]
enum MyKind {
  A,
  B,
}

#[test]
fn possible_kinds() {
  // there is a field `possible_kinds` in `Action`
  // which will be set if you use `builder.define` and `action.bind`

  // when you create a new Action, the target kind is `()`
  // so we have to use `bind` to bind the action to a specific kind
  let action = Action::<(), (), ()>::regex(r"^a")
    .unwrap()
    .bind::<MyKind>(MyKind::A);
  // `MyKind` implemented `TokenKind` so we can use `id` to get the kind's id
  // and check if the action's `possible_kinds` contains the kind's id
  assert!(action.possible_kinds().contains(&MyKind::A.id()));
  assert!(!action.possible_kinds().contains(&MyKind::B.id()));

  // when we use expectational lex, the possible kinds will be checked
  // to accelerate the lexing process
}

#[test]
fn multi_kinds() {
  // an action can be bound to multiple kinds
  // and we must provide a selector to choose a kind from the possible kinds

  let action = Action::<(), (), ()>::regex(r"^a")
    .unwrap()
    .kinds([MyKind::A, MyKind::B])
    .select(|ctx| {
      if ctx.output.rest().len() > 0 {
        MyKind::A
      } else {
        MyKind::B
      }
    });
  assert!(action.possible_kinds().contains(&MyKind::A.id()));
  assert!(action.possible_kinds().contains(&MyKind::B.id()));

  // but be aware, the possible kinds will NOT be checked during the runtime
  // so we MUST make sure the selector will always return a valid kind!

  // to use an action with possible_kinds set, we can use `builder.append` or `builder.append_from`
  let mut lexer = Builder::<MyKind, (), ()>::default()
    .append(action)
    .build("aa");

  // the first lex should be accepted as `MyKind::A`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, MyKind::A));

  // the second lex should be accepted as `MyKind::B`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, MyKind::B));

  // if you want to provide kind ids directly
  // you can use Action.kind_ids
  let action = Action::<(), (), ()>::regex(r"^a")
    .unwrap()
    .kind_ids([MyKind::A.id(), MyKind::B.id()])
    .select(|ctx| {
      if ctx.output.rest().len() > 0 {
        MyKind::A
      } else {
        MyKind::B
      }
    });
  assert!(action.possible_kinds().contains(&MyKind::A.id()));
  assert!(action.possible_kinds().contains(&MyKind::B.id()));
}
