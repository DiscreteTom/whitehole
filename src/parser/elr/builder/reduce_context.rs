use crate::{
  lexer::{token::TokenKind, trimmed::TrimmedLexer},
  parser::ast::ASTNode,
};

pub struct ReduceContext<
  'a,
  'buffer,
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  matched: &'a [usize],
  buffer: &'a Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
  reducing_stack: &'a Vec<usize>,
  lexer: &'a TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  pub data: Option<ASTData>,
  pub error: Option<ErrorType>,
}

impl<
    'a,
    'buffer,
    TKind: TokenKind<TKind> + 'static,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  >
  ReduceContext<
    'a,
    'buffer,
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >
{
  pub fn new(
    matched: &'a [usize],
    buffer: &'a Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
    reducing_stack: &'a Vec<usize>,
    lexer: &'a TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  ) -> Self {
    Self {
      matched,
      buffer,
      reducing_stack,
      lexer,
      data: None,
      error: None,
    }
  }

  pub fn matched(&self) -> &'a [usize] {
    self.matched
  }
  pub fn buffer(&self) -> &'a Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>> {
    self.buffer
  }
  pub fn reducing_stack(&self) -> &'a Vec<usize> {
    self.reducing_stack
  }
  pub fn lexer(&self) -> &'a TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType> {
    self.lexer
  }

  pub fn matched_iter(
    &self,
  ) -> impl Iterator<Item = &ASTNode<TKind, NTKind, ASTData, ErrorType, Global>> {
    self.matched.iter().map(|index| &self.buffer[*index])
  }
}

pub type Condition<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType> =
  Box<
    dyn Fn(
      &ReduceContext<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
    ) -> bool,
  >;
