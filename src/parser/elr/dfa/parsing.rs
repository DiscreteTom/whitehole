use crate::{
  lexer::{
    token::{TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::{ast::ASTNode, elr::grammar::grammar::GrammarId},
};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

use super::state::State;

pub struct Stack<T> {
  stack: Vec<T>,
}

impl<T> Clone for Stack<T>
where
  T: Clone,
{
  fn clone(&self) -> Self {
    Self {
      stack: self.stack.clone(),
    }
  }
}

impl<T> Stack<T> {
  pub fn new(stack: Vec<T>) -> Self {
    Self { stack }
  }
  pub fn push(&mut self, item: T) {
    self.stack.push(item);
  }
  pub fn pop(&mut self) -> Option<T> {
    self.stack.pop()
  }
  pub fn current(&self) -> &T {
    self.stack.last().unwrap()
  }
  pub fn clear(&mut self) {
    self.stack.clear();
  }
  pub fn truncate(&mut self, len: usize) {
    self.stack.truncate(len);
  }
}

pub struct ParsingState<
  Kind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerType,
> {
  pub buffer: Vec<ASTNode<Kind, ASTData, ErrorType, Global>>,
  pub state_stack: Stack<State<Kind, ASTData, ErrorType, Global>>,
  pub reducing_stack: Vec<usize>,
  pub lexer: LexerType,
  pub need_lex: bool,
  pub try_lex_index: usize,
  pub lexed_grammars: HashSet<GrammarId>,
  pub lexed_without_expectation: bool,
}

impl<
    'buffer,
    Kind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Clone + Default,
    LexerErrorType,
  >
  ParsingState<
    Kind,
    ASTData,
    ErrorType,
    Global,
    TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
  >
{
  pub fn try_lex(&mut self, global: &Rc<RefCell<Global>>) -> bool {
    let current_state = self.state_stack.current();

    // ensure current state has next (can digest more)
    let next = match current_state.get_next() {
      Some(next) => next,
      None => return false,
    };

    match current_state.try_lex(
      &self.lexer,
      self.try_lex_index,
      &mut self.lexed_grammars,
      &mut self.lexed_without_expectation,
      global,
    ) {
      // TODO: re-lex if the lex is not successful
      None => false,
      Some(output) => {
        // TODO: store re-lex info

        self.state_stack.push(next); // push next state to state stack
        self.reducing_stack.push(self.buffer.len()); // append new node to reducing stack
        self.buffer.push(output.node);
        self.lexer = output.lexer;
        self.need_lex = false;
        // reset lexed state since we have a new state
        self.try_lex_index = 0;
        self.lexed_grammars.clear();
        self.lexed_without_expectation = false;
        true
      }
    }
  }
}
