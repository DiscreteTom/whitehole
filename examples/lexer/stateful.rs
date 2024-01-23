use whitehole::lexer::Builder;
use whitehole_macros::TokenKind;

// define token kinds
// make sure it implements `TokenKind`
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

pub fn test() {
  let mut lexer = Builder::<MyKind, MyState, ()>::default()
    .append(|a| {
      a.regex("123")
        .unwrap()
        .bind(MyKind::A)
        // access lexer's action state by `input.state()` or `input.state_mut()`.
        // in this example we reject the action if the state's `reject` field is `true`.
        .reject_if(|ctx| ctx.input.state().reject)
        // if the action is accepted and not peek, set the state's `reject` field to `true`.
        // you can only mutate the action state in `then`.
        .then(|ctx| ctx.input.state_mut().reject = true)
    })
    // load the lexer with a buffer
    // so that the lexer can lex twice
    .build("123123");

  // by default `state.reject` is `false`
  assert_eq!(lexer.core().state().reject, false);

  // the first lex should be accepted
  assert_ne!(lexer.lex().digested, 0);

  // and the action state will be changed
  assert_eq!(lexer.core().state().reject, true);

  // then the second lex should be rejected
  assert_eq!(lexer.lex().digested, 0);
}
