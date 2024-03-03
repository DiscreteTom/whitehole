use super::{builder::lexer_panic_handler::LexerPanicHandler, dfa::dfa::Dfa};
use crate::{
  lexer::{token::TokenKind, trimmed::TrimmedLexer},
  parser::ast::ASTNode,
};
use std::{cell::RefCell, rc::Rc};

pub struct ParseOutput<
  'buffer,
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  pub buffer: Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
  pub errors: Vec<usize>,
}

pub struct Parser<
  'buffer,
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Clone + Default + 'static,
  LexerErrorType: 'static,
> {
  dfa: Rc<Dfa<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  lexer_panic_handler: LexerPanicHandler<TKind, LexerActionState, LexerErrorType>,
  global: Rc<RefCell<Global>>,
}

impl<
    'buffer,
    TKind: TokenKind<TKind> + 'static,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Clone + Default + 'static,
    LexerErrorType: 'static,
  > Clone
  for Parser<'buffer, TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  fn clone(&self) -> Self {
    Parser {
      dfa: self.dfa.clone(),
      lexer: self.lexer.clone(),
      lexer_panic_handler: self.lexer_panic_handler.clone(),
      global: self.global.clone(),
    }
  }
}

impl<
    'buffer,
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Clone + Default,
    LexerErrorType,
  > Parser<'buffer, TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  pub fn new(
    dfa: Rc<Dfa<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    lexer_panic_handler: LexerPanicHandler<TKind, LexerActionState, LexerErrorType>,
    global: Rc<RefCell<Global>>,
  ) -> Self {
    Self {
      dfa,
      lexer,
      global,
      lexer_panic_handler,
    }
  }

  pub fn parse(&mut self) -> ParseOutput<'buffer, TKind, NTKind, ASTData, ErrorType, Global> {
    self.parse_with(Vec::new())
  }

  /// Useful if you want to provision the buffer yourself.
  /// E.g. `parser.parse_with(Vec::with_capacity(100))`
  pub fn parse_with(
    &mut self,
    buffer: Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
  ) -> ParseOutput<'buffer, TKind, NTKind, ASTData, ErrorType, Global> {
    let output = self.dfa.parse(
      buffer,
      &mut self.lexer,
      &self.lexer_panic_handler,
      &self.global,
      true, // TODO: make this configurable
    );
    ParseOutput {
      buffer: output.buffer,
      errors: output.errors,
    }
  }

  pub fn reload<'new_buffer>(
    self,
    buffer: &'new_buffer str,
  ) -> Parser<
    'new_buffer,
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  > {
    Parser {
      lexer: self.lexer.reload(buffer).into(),
      ..self
    }
  }

  pub fn clone_with<'new_buffer>(
    &self,
    buffer: &'new_buffer str,
    global: Rc<RefCell<Global>>,
  ) -> Parser<
    'new_buffer,
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  > {
    Parser {
      dfa: self.dfa.clone(),
      lexer: self.lexer.clone_with_default_action_state(buffer).into(),
      lexer_panic_handler: self.lexer_panic_handler.clone(),
      global,
    }
  }
}
