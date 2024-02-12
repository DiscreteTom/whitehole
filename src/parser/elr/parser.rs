use super::dfa::dfa::Dfa;
use crate::lexer::{token::TokenKind, trimmed::TrimmedLexer};
use std::{cell::RefCell, rc::Rc};

pub struct Parser<
  'buffer,
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Clone + Default + 'static,
  LexerErrorType: 'static,
> {
  dfa: Dfa<TKind, NTKind, ASTData, ErrorType, Global>,
  lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  global: Rc<RefCell<Global>>,
}

impl<
    'buffer,
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Clone + Default,
    LexerErrorType,
  > Parser<'buffer, TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  pub fn new(
    dfa: Dfa<TKind, NTKind, ASTData, ErrorType, Global>,
    lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    global: Rc<RefCell<Global>>,
  ) -> Self {
    Self { dfa, lexer, global }
  }

  pub fn parse(&self) {
    let output = self.dfa.parse(
      Vec::new(),
      // TODO: prevent clone?
      self.lexer.clone(),
      &self.global,
    );
    // TODO
    // println!("{:?}", output.buffer);
  }
}
