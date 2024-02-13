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
}

#[test]
fn peek_lexer() {
  let mut lexer = Builder::<MyKind>::default()
    .ignore(Action::regex(r"^\s+").unwrap().bind(Anonymous))
    .define(A, Action::regex(r"a").unwrap())
    .build(" a");

  // we can peek the next token without consuming it
  // and all muted tokens are not consumed as well
  let peek = lexer.peek();
  let token = peek.token.unwrap();
  assert!(matches!(token.kind, A));
  assert_eq!(peek.digested, 2); // the whitespace is also digested

  // now use `lex` to consume the token and the muted leading whitespace
  let res = lexer.lex();
  let token = res.token.unwrap();
  assert!(matches!(token.kind, A));
  assert_eq!(res.digested, 2);

  // however, peek-then-lex is not recommended
  // because actions are evaluated twice.
  // peek will return the mutated action state and how many chars are digested
  // we can directly apply them to the lexer if the peek result is what we want
  let mut lexer = lexer.reload(" a");
  assert_eq!(lexer.state().digested(), 0);
  let peek = lexer.peek();
  lexer.take(peek.digested, Some(peek.action_state));
  assert_eq!(lexer.state().digested(), 2);

  // as you can see, peek will clone the action state
  // so there is still some overhead

  // another thing to mention is that
  // you can provide expectations when peek just like in lex
  let lexer = lexer.reload(" a");
  let peek = lexer.peek_expect("b");
  assert!(peek.token.is_none());
}

#[test]
fn trim_lexer() {
  // if you want to peek different kinds of tokens
  // and there are muted tokens in the beginning of the rest of the buffer
  // then the muted tokens will be lexed multi times
  // which is not efficient

  let mut lexer = Builder::<MyKind>::default()
    .ignore(Action::regex(r"^\s+").unwrap().bind(Anonymous))
    .define(A, Action::regex(r"a").unwrap())
    .define(B, Action::regex(r"a").unwrap())
    .build(" a");

  // for example, this peek will first ignore the whitespace then yield `A`
  let peek = lexer.peek_expect(&A);
  assert!(matches!(peek.token.unwrap().kind, A));
  assert_eq!(peek.digested, 2);

  // if then we do another peek with different expectation
  // the lexer will ignore the whitespace again
  let peek = lexer.peek_expect(&B);
  assert!(matches!(peek.token.unwrap().kind, B));
  assert_eq!(peek.digested, 2);

  // to avoid duplicated lexing, we can first trim the lexer to remove the muted tokens
  // then peek the rest of the buffer
  lexer.trim(); // this will consume the whitespace
  assert_eq!(lexer.state().digested(), 1);
  // now we can peek the rest of the buffer
  let peek = lexer.peek_expect(&A);
  assert!(matches!(peek.token.unwrap().kind, A));
  assert_eq!(peek.digested, 1);
  let peek = lexer.peek_expect(&B);
  assert!(matches!(peek.token.unwrap().kind, B));
  assert_eq!(peek.digested, 1);

  // for strict typing, we also have a `TrimmedLexer` struct,
  // you can use `lexer.into_trimmed` to consume the lexer and get the `TrimmedLexer`.
  let res = lexer.reload(" a").into_trimmed();
  assert_eq!(res.digested, 1);
  // get the trimmed lexer
  let trimmed_lexer = res.trimmed_lexer;
  let peek = trimmed_lexer.peek();
  assert!(matches!(peek.token.unwrap().kind, A));
  assert_eq!(peek.digested, 1);
}
