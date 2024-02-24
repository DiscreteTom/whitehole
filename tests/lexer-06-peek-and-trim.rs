use whitehole::lexer::{trimmed::TrimmedLexer, Action, LexerBuilder};
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
  let mut lexer = LexerBuilder::<MyKind>::default()
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

  let mut lexer = LexerBuilder::<MyKind>::default()
    .ignore(Action::regex(r"^\s+").unwrap().bind(Anonymous))
    .define(A, Action::regex(r"^a").unwrap())
    .define(B, Action::regex(r"^a").unwrap())
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
}

#[test]
fn trimmed_lexer() {
  let lexer = LexerBuilder::<MyKind>::default()
    .ignore(Action::regex(r"^\s+").unwrap().bind(Anonymous))
    .define(A, Action::regex(r"^(a|b|c)").unwrap())
    .build(" a b c");

  // for strict typing, we also have a `TrimmedLexer` struct
  let mut trimmed_lexer: TrimmedLexer<_, _, _> = lexer.into();

  // once the lexer is transformed into a trimmed lexer, the lexer is trimmed
  assert!(trimmed_lexer.state().trimmed());
  assert_eq!(trimmed_lexer.state().digested(), 1);

  // trimmed lexer's lex and take methods are similar to the lexer's
  // but after that the lexer will be trimmed
  let (lex_output, trim_output) = trimmed_lexer.lex();
  assert_eq!(lex_output.digested, 1); // digest the `a`
  assert_eq!(trim_output.digested, 1); // digest the whitespace after `a`
  assert!(trimmed_lexer.state().trimmed());
  // take `b` from the buffer
  let (_, trim_output) = trimmed_lexer.take(1, None);
  assert_eq!(trim_output.digested, 1); // digest the whitespace after `b`
  assert!(trimmed_lexer.state().trimmed());
}
