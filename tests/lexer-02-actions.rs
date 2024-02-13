use whitehole::lexer::{Action, Builder};
use whitehole_macros::TokenKind;
use MyKind::*; // use the enum variants directly

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone)]
enum MyKind {
  Anonymous,
  A,
  B,
  C,
  D,
}

// define your custom action state
// make sure it implements `Default` and `Clone`
#[derive(Default, Clone)]
struct MyState {
  reject: bool,
}

#[test]
fn action_orders() {
  let mut lexer = Builder::<MyKind, MyState>::default()
    // first defined actions have higher priority
    .define(A, Action::regex(r"^a").unwrap())
    .define(B, Action::regex(r"^a").unwrap())
    // different actions can share the same target token kind
    .define(A, Action::regex(r"^b").unwrap())
    .build("ab");

  // the first lex should be accepted as `A`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, A));

  // the second lex should be accepted as `A` too
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, A));
}

#[test]
fn action_decorators() {
  let mut lexer = Builder::<MyKind, MyState, &str>::default()
    // when using action decorators
    // rust compiler can't infer the action's generic parameters
    // so we need to use `define_with`, `append_with` and `ignore_with` to define actions
    // these methods accept a function which takes an `ActionBuilder` as its parameter
    // so the action's generic parameters can be inferred from the `ActionBuilder`
    .define_with(Anonymous, |a| {
      // to mute an action, we can use `mute` or `mute_if`
      a.regex(r"^\s+").unwrap().mute(true)
    })
    .define_with(A, |a| {
      // to set token's error, we can use `check` or `error`
      a.regex(r"^a").unwrap().check(|ctx| {
        if ctx.output.rest().len() > 0 {
          Some("error")
        } else {
          None
        }
      })
    })
    .define_with(B, |a| {
      // to reject an action after the output is yielded, we can use `reject` or `reject_if`
      a.regex(r"^b")
        .unwrap()
        .reject_if(|ctx| ctx.output.rest().len() > 0)
    })
    .define_with(C, |a| {
      // to reject an action before the output is yielded, we can use `prevent`
      a.regex(r"^c")
        .unwrap()
        .prevent(|input| input.state().reject)
    })
    .define_with(D, |a| {
      // use `then` to run a callback if this action is accepted and is not a peek
      // this is usually used to modify lexer's action state
      a.regex(r"^d")
        .unwrap()
        .then(|ctx| {
          ctx.input.state_mut().reject = true;
        })
        // yes you can apply multi decorators to an action
        .prevent(|input| input.state().reject)
    })
    .build("a b c");

  // the first lex should be accepted but with error set
  let res = lexer.lex();
  let token = res.token.unwrap();
  assert!(matches!(token.kind, A));
  assert!(matches!(token.error, Some("error")));
  assert_eq!(res.digested, 1);
  // res.token is not included in res.errors even if the token has error
  assert_eq!(res.errors.len(), 0);

  // the second lex should be rejected but still digest some characters
  let res = lexer.lex();
  assert!(matches!(res.token, None));
  assert_eq!(res.digested, 1); // digest one whitespace
  assert_eq!(res.errors.len(), 0); // no new error

  // create a new lexer with the same actions and a new input
  let mut lexer = lexer.reload("c d c");

  // the first lex should be accepted
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, C));
  assert_eq!(token.range.start, 0);
  assert_eq!(token.range.end, 1);

  // the second lex should be accepted and will change the state
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind, D));
  assert_eq!(token.range.start, 2);
  assert_eq!(token.range.end, 3);
  assert_eq!(lexer.action_state().reject, true);

  // the third lex should be rejected
  let res = lexer.lex();
  assert!(matches!(res.token, None));
}
