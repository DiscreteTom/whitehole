use whitehole::lexer::Builder;
use whitehole_macros::TokenKind;

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone)]
enum MyKind {
  A,
}

// define your custom action state
// make sure it implements `Default` and `Clone`
#[derive(Default, Clone)]
struct MyState {
  reject: bool,
}

#[test]
fn stateful_lexer() {
  let mut lexer = Builder::<MyKind, MyState, ()>::default()
    .append_from(|a| {
      a.regex("^123")
        .unwrap()
        .bind(MyKind::A)
        // access lexer's action state by `input.state()` or `input.state_mut()`.
        // in this example we reject the action if the state's `reject` field is `true`.
        .prevent(|input| input.state().reject)
        // if the action is accepted, set the state's `reject` field to `true`.
        .then(|ctx| ctx.input.state_mut().reject = true)
    })
    // with this input text the lexer can lex twice
    .build("123123");

  // by default `state.reject` is `false`
  assert_eq!(lexer.action_state().reject, false);

  // the first lex should be accepted
  assert_ne!(lexer.lex().digested, 0);

  // and the action state will be changed
  assert_eq!(lexer.action_state().reject, true);

  // then the second lex should be rejected
  assert_eq!(lexer.lex().digested, 0);

  // besides, you can set the action_state directly
  lexer.action_state_mut().reject = false;
  assert_eq!(lexer.action_state().reject, false);
  *lexer.action_state_mut() = MyState { reject: true };
  assert_eq!(lexer.action_state().reject, true);
}
