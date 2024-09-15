use whitehole::{
  kind::{kind, SubKind},
  lexer::{
    action::{regex, whitespaces},
    builder::LexerBuilder,
  },
};

// define token kinds, make sure it is decorated by `#[kind]`
#[kind]
#[derive(Clone, Default)]
enum MyKind {
  #[default]
  Anonymous,
  A,
  B,
}

#[test]
fn peek_lexer() {
  let mut lexer = LexerBuilder::new()
    .ignore_default(whitespaces())
    .define(A, regex(r"a"))
    .define(B, regex(r"b"))
    .build(" a");

  // we can peek the next token without updating the lexer's state,
  let (output, _) = lexer.peek();
  let token = output.token.unwrap();
  assert!(matches!(token.binding.kind(), MyKind::A));
  assert_eq!(output.digested, 2); // the whitespace is also digested by the peek
  assert_eq!(lexer.instant().digested(), 0); // but the lexer's state is not updated

  // now use `lex` to consume the token and the muted leading whitespace
  let res = lexer.lex();
  let token = res.token.unwrap();
  assert!(matches!(token.binding.kind(), MyKind::A));
  assert_eq!(res.digested, 2);

  // however, peek-then-lex is not recommended
  // because actions will be evaluated twice.
  // peek will return the mutated state and how many chars are digested
  // we can directly apply them to the lexer if the peek result is what we want
  let mut lexer = lexer.reload(" a");
  assert_eq!(lexer.instant().digested(), 0);
  let (output, new_state) = lexer.peek();
  lexer.digest_with(output.digested, new_state);
  assert_eq!(lexer.instant().digested(), 2);

  // as you can see, peek will clone the state
  // so there is still some overhead

  // another thing to mention is that
  // you can provide expectations when peek just like in lex
  let mut lexer = lexer.reload(" a");
  let (output, _) = lexer.peek_with(|o| o.expect(B::kind_id()));
  assert!(output.token.is_none());
}

#[test]
fn peek_vs_clone() {
  // you can also clone the lexer to realize the same effect,
  // the difference is that when cloning a lexer, the LexerState will also be cloned,
  // and a new lexer will be created.
  // cloning a LexerState is not very expensive, so the costs between peek and clone are similar.
  // if you just want to peek the next token, use peek.
  // if you want to store the lexer's state and continue from there, use clone.

  let lexer = LexerBuilder::new()
    .ignore_default(whitespaces())
    .define(A, regex(r"a"))
    .build(" a");

  let mut cloned = lexer.clone();
  cloned.lex();

  assert_eq!(lexer.instant().digested(), 0);
}

#[test]
fn trim_lexer() {
  // if you want to peek different kinds of tokens
  // and there are muted tokens at the beginning of the rest of the buffer
  // then the muted tokens will be lexed multi times
  // which is not efficient

  let mut lexer = LexerBuilder::new()
    .ignore_default(whitespaces())
    .define(A, regex(r"^a"))
    .define(B, regex(r"^a"))
    .build(" a");

  // for example, this peek will first ignore the whitespace then yield `A`
  let (output, _) = lexer.peek_with(|o| o.expect(A::kind_id()));
  assert!(matches!(output.token.unwrap().binding.kind(), MyKind::A));
  assert_eq!(output.digested, 2);

  // if then we do another peek with different expectation
  // the lexer will ignore the whitespace again
  let (output, _) = lexer.peek_with(|o| o.expect(B::kind_id()));
  assert!(matches!(output.token.unwrap().binding.kind(), MyKind::B));
  assert_eq!(output.digested, 2);

  // to prevent lexing the whitespaces multiple times
  // we can first trim the lexer
  // which will lex the lexer with all muted actions
  let res = lexer.trim().unwrap();
  assert_eq!(res.digested, 1); // only the whitespace is digested
  assert_eq!(lexer.instant().digested(), 1);
  assert!(lexer.instant().trimmed()); // lexer will also record whether it is already trimmed
  assert!(lexer.trim().is_none()); // trim the lexer again, the lexer is already trimmed, so it will return None

  // now if we peek the lexer, the whitespaces won't be lexed again
  let (output, _) = lexer.peek_with(|o| o.expect(A::kind_id()));
  assert!(matches!(output.token.unwrap().binding.kind(), MyKind::A));
  assert_eq!(output.digested, 1); // only the 'a' is digested
  let (output, _) = lexer.peek_with(|o| o.expect(B::kind_id()));
  assert!(matches!(output.token.unwrap().binding.kind(), MyKind::B));
  assert_eq!(output.digested, 1); // only the 'a' is digested
}
