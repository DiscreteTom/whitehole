use super::dfa::{dfa::Dfa, parsing::Stack, state::State};
use crate::{
  lexer::{token::TokenKind, trimmed::TrimmedLexer},
  parser::ast::ASTNode,
};
use std::{cell::RefCell, rc::Rc};

pub enum ParseContinuable<StateStack> {
  Yes(StateStack),
  No,
}

pub struct ParseOutput<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  pub buffer: Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
  pub errors: Vec<usize>,
  pub continuable: ParseContinuable<
    Stack<Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>>,
  >,
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
  dfa: Dfa<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
  lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  global: Rc<RefCell<Global>>,
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
    dfa: Dfa<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
    lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    global: Rc<RefCell<Global>>,
  ) -> Self {
    Self { dfa, lexer, global }
  }

  pub fn parse(
    &mut self,
  ) -> ParseOutput<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType> {
    self.parse_with(
      Vec::new(),
      Stack::new(vec![self.dfa.entry_state().clone()]),
      [],
    )
  }

  // TODO: better name
  pub fn parse_continue(
    &mut self,
    buffer: Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
    state_stack: Stack<
      Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    >,
  ) -> ParseOutput<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType> {
    let last = buffer.len() - 1;
    self.parse_with(buffer, state_stack, [last])
  }

  pub fn parse_with(
    &mut self,
    buffer: Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
    state_stack: Stack<
      Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    >,
    reducing_stack: impl Into<Vec<usize>>,
  ) -> ParseOutput<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType> {
    let output = self.dfa.parse(
      buffer,
      state_stack,
      reducing_stack.into(),
      // TODO: prevent clone?
      self.lexer.clone(),
      &self.global,
    );
    self.lexer = output.lexer;
    ParseOutput {
      buffer: output.buffer,
      errors: output.errors,
      continuable: output.continuable,
    }
  }

  pub fn parse_all(
    &mut self,
  ) -> ParseOutput<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType> {
    todo!()
  }
}
