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
fn peek_lexer() {
  let mut lexer = Builder::<MyKind, (), ()>::default()
    .ignore(Action::regex(r"^\s+").unwrap().bind(MyKind::Anonymous))
    .define(MyKind::A, Action::regex(r"a").unwrap())
    .build(" a");

  // we can peek the next token without consuming it
  // and all muted tokens are not consumed as well
  let peek = lexer.peek();
  let token = peek.token.unwrap();
  assert!(matches!(token.kind(), MyKind::A));
  // now use `lex` to consume the token and the muted leading whitespace
  let res = lexer.lex();
  let token = res.token.unwrap();
  assert!(matches!(token.kind(), MyKind::A));
  assert_eq!(res.digested, 2);

  // however, peek-then-lex is not recommended
  // because actions are evaluated twice.
  // peek will return the mutated action state and how many chars are digested
  // we can directly apply them to the lexer if the peek result is what we want
  let mut lexer = lexer.dry_clone(" a");
  let peek = lexer.peek();
  lexer.take(peek.digested, Some(peek.state));
  assert_eq!(lexer.state().digested(), 2);

  // as you can see, peek will clone the action state
  // so there is still some overhead

  // another thing to mention is that
  // you can provide expectations when peek just like in lex
  let lexer = lexer.dry_clone(" a");
  let peek = lexer.peek_expect("b");
  assert!(peek.token.is_none());
}
