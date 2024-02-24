use crate::{
  lexer::{
    token::{Token, TokenKind},
    trimmed::TrimmedLexer,
  },
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
  matched_indexes: &'a [usize],
  buffer: &'a Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
  reducing_stack: &'a Vec<usize>,
  next_token: &'a Option<Token<'buffer, TKind, LexerErrorType>>,
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
    matched_indexes: &'a [usize],
    buffer: &'a Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
    reducing_stack: &'a Vec<usize>,
    next_token: &'a Option<Token<'buffer, TKind, LexerErrorType>>,
    lexer: &'a TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  ) -> Self {
    Self {
      matched_indexes,
      buffer,
      reducing_stack,
      next_token,
      lexer,
      data: None,
      error: None,
    }
  }

  pub fn matched_indexes(&self) -> &'a [usize] {
    self.matched_indexes
  }
  pub fn buffer(&self) -> &'a Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>> {
    self.buffer
  }
  pub fn reducing_stack(&self) -> &'a Vec<usize> {
    self.reducing_stack
  }
  pub fn next_token(&self) -> &'a Option<Token<'buffer, TKind, LexerErrorType>> {
    self.next_token
  }
  /// The lexer after lex the [`ReduceContext::next_token`].
  pub fn lexer(&self) -> &'a TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType> {
    self.lexer
  }

  /// Get the matched node with the index.
  pub fn matched(
    &self,
    index: usize,
  ) -> &ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global> {
    &self.buffer[self.matched_indexes[index]]
  }
  /// Get an iterator of the matched nodes.
  pub fn matched_iter(
    &self,
  ) -> impl Iterator<Item = &ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>> {
    self
      .matched_indexes
      .iter()
      .map(|index| &self.buffer[*index])
  }
  /// Get the `ASTData`'s value of the matched node with the index.
  /// Shortcut for `self.matched(index).data.as_ref().unwrap()`.
  pub fn values(&self, index: usize) -> &ASTData {
    self.matched(index).data.as_ref().unwrap()
  }
}

pub type Condition<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType> =
  Box<
    dyn Fn(
      &ReduceContext<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
    ) -> bool,
  >;
pub type Callback<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType> =
  Box<
    dyn Fn(
      &mut ReduceContext<
        TKind,
        NTKind,
        ASTData,
        ErrorType,
        Global,
        LexerActionState,
        LexerErrorType,
      >,
    ),
  >;
