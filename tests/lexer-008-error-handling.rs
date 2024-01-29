use whitehole::lexer::{Action, Builder};
use whitehole_macros::TokenKind;

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
  let lexer = Builder::<MyKind, (), &str>::default()
    .ignore_from(|a| {
      a.regex(r"^\s+")
        .unwrap()
        .bind(MyKind::Anonymous)
        // set error by using `error`
        .error("ignored")
    })
    .define_from(MyKind::A, |a| {
      // set error by using `check`
      a.regex(r"^a").unwrap().check(|ctx| {
        if ctx.output.rest().len() == 0 {
          Some("end")
        } else {
          None
        }
      })
    })
    .build(" a");

  // when `lex`, `peek` or `trim`, we can get error tokens from the output
  let peek = lexer.peek();
  // now even if the whitespace is muted, it contains an error
  // so we can get the error token from the peek result
  // be ware: `errors` in the result doesn't contain the peeked token
  assert_eq!(peek.errors.len(), 1);
  assert!(matches!(peek.errors[0].kind(), MyKind::Anonymous));
  assert!(matches!(peek.errors[0].error(), Some("ignored")));
  // we can still get the peeked (error) token
  let token = peek.token.unwrap();
  assert!(matches!(token.kind(), MyKind::A));
  assert!(matches!(token.error(), Some("end")));
}

#[test]
fn panic_mode() {
  // for some unacceptable errors, we can use panic mode
  // which will digest 1 char and try again

  let mut lexer = Builder::<MyKind, (), &str>::default()
    .ignore(Action::regex(r"^\s+").unwrap().bind(MyKind::Anonymous))
    .define(MyKind::A, Action::regex(r"^a").unwrap())
    .build("b a");

  // in this case when we peek the lexer
  // the 'b' is not accepted by any action
  // and the peek will fail
  let peek = lexer.peek();
  assert!(peek.token.is_none());
  assert_eq!(peek.digested, 0);

  // enter panic mode, take 1 char and try again
  // this will reset the lexer's action state, unless we provide a new state
  lexer.take(1, None);
  // now we can peek
  let peek = lexer.peek();
  assert!(matches!(peek.token.unwrap().kind(), MyKind::A));
  assert_eq!(peek.digested, 2);

  // further more, if you know what you are doing
  // you can take more chars and manually set the action state
}
