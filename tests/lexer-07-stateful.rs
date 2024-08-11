use whitehole::lexer::{action::regex, token::token_kind, LexerBuilder};

// define token kinds, make sure it is decorated by `#[token_kind]`
#[token_kind]
#[derive(Clone, Default)]
enum MyKind {
  #[default]
  Anonymous,
  A,
}

// define your custom state
#[derive(Default, Clone)]
struct MyState {
  reject: bool,
}

#[test]
fn stateful_lexer() {
  let mut lexer = LexerBuilder::stateful::<MyState>()
    .define_with(A, regex("^123"), |a| {
      a
        // access lexer's state by `input.state`.
        // in this example we reject the action if the state's `reject` field is `true`.
        .prevent(|input| input.state.reject)
        // if the action is accepted, set the state's `reject` field to `true`.
        .callback(|ctx| ctx.input.state.reject = true)
    })
    // with this input text the lexer can lex twice
    .build("123123");

  // by default `state.reject` is `false`
  assert_eq!(lexer.state.reject, false);

  // the first lex should be accepted
  assert_ne!(lexer.lex().digested, 0);

  // and the state will be changed
  assert_eq!(lexer.state.reject, true);

  // then the second lex should be rejected
  assert_eq!(lexer.lex().digested, 0);

  // besides, you can mutate or set the state directly
  lexer.state.reject = false;
  assert_eq!(lexer.state.reject, false);
  lexer.state = MyState { reject: true };
  assert_eq!(lexer.state.reject, true);
}
