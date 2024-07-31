use whitehole::lexer::{
  action::{regex, whitespaces},
  token::{token_kind, Range},
  LexerBuilder,
};

// define token kinds, make sure it is decorated by `#[token_kind]`
#[token_kind]
#[derive(Clone, Default)]
enum MyKind {
  #[default]
  Anonymous,
  A,
}

#[test]
fn error_tokens() {
  // tokens can have errors,
  // these are acceptable errors and won't stop the lexing loop

  // in this example we use `&str` as the error type.
  // use `LexerBuilder::with_error` to auto infer the error type
  let lexer = LexerBuilder::with_error()
    .ignore_default_with(whitespaces(), |a| {
      a
        // set the error by using `error`
        .error("ignored")
    })
    .define_with(A, regex(r"^a"), |a| {
      // set the error by using `check`
      a.check(|ctx| {
        if ctx.rest().len() == 0 {
          Some("end")
        } else {
          None
        }
      })
    })
    .build(" a");

  // when `lex`, `peek` or `trim`, we can get error tokens from the output
  let (output, _) = lexer.peek_with(|o| o.errors_to_vec());
  // even if the whitespace is muted, it contains an error.
  // errors will be collected with its range and error value
  assert_eq!(output.errors.len(), 2);
  assert!(matches!(
    output.errors[0],
    ("ignored", Range { start: 0, end: 1 })
  ));
  // we can still get the peeked token, which also contains an error
  // but it is not muted
  let token = output.token.unwrap();
  assert!(matches!(token.kind.value(), MyKind::A));
  assert!(matches!(
    output.errors[1],
    ("end", Range { start: 1, end: 2 })
  ));
}

#[test]
fn panic_mode() {
  // for some unacceptable errors, you can use "panic mode"
  // which will digest 1 char and try again

  let mut lexer = LexerBuilder::new()
    .ignore_default(regex(r"^\s+"))
    .define(A, regex(r"^a"))
    .build("b a");

  // in this case when we peek the lexer
  // the 'b' is not accepted by any action
  // and the peek will fail
  let (output, _) = lexer.peek();
  assert!(output.token.is_none());
  assert_eq!(output.digested, 0);

  // enter panic mode, digest 1 char directly and try again
  // this will reset the lexer's action state, unless you provide a new action state
  lexer.digest(1);
  // now we can peek
  let (output, _) = lexer.peek();
  assert!(matches!(output.token.unwrap().kind.value(), MyKind::A));
  assert_eq!(output.digested, 2);

  // further more, if you know what you are doing
  // you can take more chars and manually set the action state
}
