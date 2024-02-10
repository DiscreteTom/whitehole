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

use super::state::{State, StateId};

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
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerType,
> {
  pub buffer: Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
  pub state_stack: Stack<Rc<State<TKind, NTKind, ASTData, ErrorType, Global>>>,
  pub reducing_stack: Vec<usize>,
  pub lexer: LexerType,
  pub need_lex: bool,
  pub try_lex_index: usize,
  pub lexed_grammars: HashSet<GrammarId>,
  pub lexed_without_expectation: bool,
  pub errors: Vec<usize>,
}

impl<
    'buffer,
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Clone + Default,
    LexerErrorType,
  >
  ParsingState<
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  >
{
  pub fn try_lex(
    &mut self,
    states: &HashMap<StateId, Rc<State<TKind, NTKind, ASTData, ErrorType, Global>>>,
    global: &Rc<RefCell<Global>>,
  ) -> bool {
    let current_state = self.state_stack.current();

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

        let node_index = self.buffer.len();
        if output.node.error.is_some() {
          self.errors.push(node_index);
        }
        self
          .state_stack
          .push(states.get(&output.next_state_id).unwrap().clone()); // push next state to state stack
        self.reducing_stack.push(node_index); // append new node to reducing stack
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

  pub fn try_reduce(
    &mut self,
    entry_nts: &HashSet<TokenKindId>,
    follow_sets: &HashMap<TokenKindId, TokenKindId>,
    states: &HashMap<StateId, Rc<State<TKind, NTKind, ASTData, ErrorType, Global>>>,
  ) -> bool {
    let output = match self.state_stack.current().try_reduce(
      &mut self.buffer,
      &self.lexer,
      &mut self.reducing_stack,
      entry_nts,
      follow_sets,
    ) {
      None => {
        // reduce failed, try to lex more
        self.need_lex = true;
        return false;
      }
      Some(output) => output,
    };

    // reduce success
    // link children's parent
    let node_index = self.buffer.len();
    output
      .node
      .children
      .iter()
      .for_each(|i| self.buffer[*i].parent = Some(node_index));

    // push new node to buffer
    self.buffer.push(output.node);

    // reduced n nodes, generate 1 node
    self
      .reducing_stack
      .truncate(self.reducing_stack.len() - output.reduced);
    self.reducing_stack.push(node_index);

    // remove the reduced states, push the new state
    self.state_stack.truncate(output.reduced);
    self
      .state_stack
      .push(states.get(&output.next_state_id).unwrap().clone());
    true
  }
}
