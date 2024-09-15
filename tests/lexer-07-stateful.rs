use whitehole::{
  kind::whitehole_kind,
  lexer::{action::regex, builder::LexerBuilder},
};

// define token kinds, make sure it is decorated by `#[whitehole_kind]`
#[whitehole_kind]
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
        .then(|ctx| ctx.input.state.reject = true)
    })
    // with this input text the lexer can lex twice
    .build("123123");

  // by default `state.reject` is `false`
  assert!(!lexer.state.reject);

  // the first lex should be accepted
  assert_ne!(lexer.lex().digested, 0);

  // and the state will be changed
  assert!(lexer.state.reject);

  // then the second lex should be rejected
  assert_eq!(lexer.lex().digested, 0);

  // besides, you can mutate or set the state directly
  lexer.state.reject = false;
  assert!(!lexer.state.reject);
  lexer.state = MyState { reject: true };
  assert!(lexer.state.reject);
}
