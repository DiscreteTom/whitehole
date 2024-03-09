use whitehole::lexer::{action::regex, LexerBuilder};
use whitehole_macros::TokenKind;
use MyKind::*; // use the enum variants directly

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone)]
enum MyKind {
  Anonymous,
  A,
}

#[test]
fn error_tokens() {
  // we can specify the error type as the 3rd generic parameter in `Builder`
  // this is acceptable errors, the lexing process won't stop

  // in this example we use `&str` as the error type
  let lexer = LexerBuilder::<MyKind, (), &str>::default()
    .ignore_with(regex(r"^\s+").unwrap().bind(Anonymous), |a| {
      a
        // set error by using `error`
        .error("ignored")
        .into()
    })
    .define_with(A, regex(r"^a").unwrap(), |a| {
      // set error by using `check`
      a.check(|ctx| {
        if ctx.output.rest().len() == 0 {
          Some("end")
        } else {
          None
        }
      })
      .into()
    })
    .build(" a");

  // when `lex`, `peek` or `trim`, we can get error tokens from the output
  let (output, _) = lexer.peek();
  // now even if the whitespace is muted, it contains an error
  // so we can get the error token from the peek result
  // be ware: `errors` in the result doesn't contain the peeked token
  assert_eq!(output.errors.len(), 1);
  assert!(matches!(output.errors[0].kind, Anonymous));
  assert!(matches!(output.errors[0].error, Some("ignored")));
  // we can still get the peeked (error) token
  let token = output.token.unwrap();
  assert!(matches!(token.kind, A));
  assert!(matches!(token.error, Some("end")));
}

#[test]
fn panic_mode() {
  // for some unacceptable errors, we can use panic mode
  // which will digest 1 char and try again

  let mut lexer = LexerBuilder::<MyKind, (), &str>::default()
    .ignore(regex(r"^\s+").unwrap().bind(Anonymous))
    .define(A, regex(r"^a").unwrap())
    .build("b a");

  // in this case when we peek the lexer
  // the 'b' is not accepted by any action
  // and the peek will fail
  let (output, _) = lexer.peek();
  assert!(output.token.is_none());
  assert_eq!(output.digested, 0);

  // enter panic mode, take 1 char and try again
  // this will reset the lexer's action state, unless we provide a new state
  lexer.take(1);
  // now we can peek
  let (output, _) = lexer.peek();
  assert!(matches!(output.token.unwrap().kind, A));
  assert_eq!(output.digested, 2);

  // further more, if you know what you are doing
  // you can take more chars and manually set the action state
}
