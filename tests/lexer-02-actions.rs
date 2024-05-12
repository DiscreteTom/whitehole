use whitehole::lexer::{action::regex, token::token_kind, LexerBuilder};

// define token kinds, make sure it is decorated by `#[token_kind]`
#[token_kind]
#[derive(Clone, Default)]
enum MyKind {
  #[default]
  Anonymous,
  A,
  B,
  C,
  D,
}

#[test]
fn action_orders() {
  let mut lexer = LexerBuilder::new()
    // first defined actions have higher priority
    .define(A, regex(r"^.")) // highest priority
    .define(B, regex(r"^."))
    .define(C, regex(r"^.")) // lowest priority
    .build("aa");

  // lexing will always emit `A`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind.value(), MyKind::A));
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind.value(), MyKind::A));
}

// define action state which can be shared between actions
#[derive(Default, Clone)]
struct MyState {
  reject: bool,
}

#[test]
fn action_decorators() {
  let mut lexer = LexerBuilder::stateful_with_error::<MyState>()
    // you can use `define_with` to apply a decorator to all the actions in the `define` call
    .define_with(
      Anonymous,
      // to mute an action (just like `LexerBuilder::ignore`), you can use `mute`
      regex(r"^\s+"),
      |a| a.mute(),
    )
    .define_with(
      A,
      // to set token's error, you can use `check` or `error`
      regex(r"^a"),
      |a| a.error("error"),
    )
    .define_with(
      B,
      // to reject an action after the action is executed and accepted, you can use `reject` or `reject_if`
      regex(r"^b"),
      |a| a.reject_if(|ctx| ctx.rest().len() > 0),
    )
    .define_with(
      C,
      // to reject an action before the action is executed, you can use `prevent`
      regex(r"^c"),
      |a| a.prevent(|input| input.state.reject),
    )
    .define_with(
      D,
      // use `then` to run a callback if this action is accepted
      // this is usually used to modify lexer's action state
      regex(r"^d"),
      |a| {
        a.callback(|ctx| {
          ctx.input.state.reject = true;
        })
        // yes you can apply multi decorators to an action
        .prevent(|input| input.state.reject)
      },
    )
    .build("a b c");

  // the first lex should be accepted but with error set
  let res = lexer.lex();
  let token = res.token.unwrap();
  assert!(matches!(token.kind.value(), MyKind::A));
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
  assert!(matches!(token.kind.value(), MyKind::C));
  assert_eq!(token.range.start, 0);
  assert_eq!(token.range.end, 1);

  // the second lex should be accepted and will change the state
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.kind.value(), MyKind::D));
  assert_eq!(token.range.start, 2);
  assert_eq!(token.range.end, 3);
  assert_eq!(lexer.action_state.reject, true);

  // the third lex should be rejected
  let res = lexer.lex();
  assert!(matches!(res.token, None));
}
