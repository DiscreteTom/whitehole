use super::state::{State, StateId, StatefulState};
use crate::{
  lexer::{
    token::{Token, TokenKind},
    trimmed::TrimmedLexer,
  },
  parser::{
    ast::ASTNode,
    elr::grammar::grammar::{Grammar, GrammarId},
  },
};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

// TODO: move to utils?
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
  pub fn current(&mut self) -> &mut T {
    self.stack.last_mut().unwrap()
  }
  pub fn clear(&mut self) {
    self.stack.clear();
  }
  pub fn truncate(&mut self, len: usize) {
    self.stack.truncate(len);
  }
  pub fn pop_n(&mut self, n: usize) {
    self.stack.truncate(self.stack.len() - n);
  }
}

pub struct ParsingState<
  'buffer,
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  // TODO: type alias for buffer, state_stack, etc
  pub buffer: Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
  pub state_stack: Stack<
    StatefulState<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
  >,
  pub reducing_stack: Vec<usize>,
  pub lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  /// `None` if not ready, `Some(None)` if EOF, `Some(Some(token))` if next token exists.
  pub next_token: Option<Option<Token<'buffer, TKind, LexerErrorType>>>,
  pub need_lex: bool,
  pub errors: Vec<usize>,
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
  >
  ParsingState<'buffer, TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  pub fn try_lex(
    &mut self,
    states: &HashMap<
      StateId,
      Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    >,
    global: &Rc<RefCell<Global>>,
  ) -> bool {
    let current_state = self.state_stack.current();

    match current_state.try_lex(&self.lexer, global) {
      // TODO: re-lex if the lex is not successful
      None => false,
      Some(output) => {
        // TODO: store re-lex info

        let node_index = self.buffer.len();
        if output.node.error.is_some() {
          self.errors.push(node_index);
        }
        // push next state to state stack
        self
          .state_stack
          .push(states.get(&output.next_state_id).unwrap().clone().into());
        self.reducing_stack.push(node_index); // append new node to reducing stack
        self.buffer.push(output.node);
        self.lexer = output.lexer;
        self.need_lex = false;
        true
      }
    }
  }

  pub fn try_reduce(
    &mut self,
    entry_nts: &HashSet<GrammarId>,
    follow_sets: &HashMap<GrammarId, HashSet<Rc<Grammar<TKind, NTKind>>>>,
    states: &HashMap<
      StateId,
      Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    >,
    global: &Rc<RefCell<Global>>,
  ) -> TryReduceResult {
    // before try reduce, we need to make sure next token is ready
    if self.next_token.is_none() {
      // here we can't use `self.try_lex` to get next token
      // because `try_lex` will use current state's candidate to lex
      // but that's not what we want.
      // e.g. we have `S := A B`, `A := C` and we are trying to reduce `C := c`,
      // `self.try_lex` will use `C := c` to lex but the lex is already done and we want to reduce.
      // actually we want to lex using B's expectation in `S := A B` because it is the *final* reduce target
      // but we don't know if the reduce will success or not
      // so this lex is not expectational,
      // which means expectational lex after an NT is not working.
      // in our case, we can't lex `B` expectational because it is after `A`
      // TODO: panic if user want expectational lex after NT
      // TODO: prevent the clone of lexer
      let output = self.lexer.clone().lex();
      self.lexer = output.lexer.into(); // trim the lexer
      match output.token {
        None => {
          self.next_token = Some(None);
        }
        Some(token) => {
          self.next_token = Some(Some(token));
        }
      }
    }

    // the next_token will be used by multi iterations of the loop
    loop {
      let output = match self.state_stack.current().try_reduce(
        &mut self.buffer,
        self.next_token.as_ref().unwrap(),
        &self.lexer,
        &mut self.reducing_stack,
        entry_nts,
        follow_sets,
      ) {
        None => {
          // reduce failed, try to lex more
          self.need_lex = true;
          // reset next_token, append buffer if next_token exists
          if let Some(token) = self.next_token.take().unwrap() {
            // push next state to state stack if the token is accepted by the current state
            if let Some(next_state_id) = self
              .state_stack
              .current()
              .try_accept_t_node_without_expectation(&token.kind.id(), token.content)
            {
              let next_state = states.get(next_state_id).unwrap().clone().into();
              self.state_stack.push(next_state);
            } else {
              // current state can't accept the token, enter panic mode just like try_lex failed
              // TODO: do we need to restore lexer state?
              return TryReduceResult::EnterPanicMode;
            }
            let node_index = self.buffer.len();
            self.reducing_stack.push(node_index); // append new node to reducing stack
            self.buffer.push(ASTNode::new_t(
              token.kind,
              token.content,
              token.range,
              global.clone(),
              None,
              None,
            ));
            return TryReduceResult::NeedLex;
          }
          // no next token and reduce failed, enter panic mode
          return TryReduceResult::EnterPanicMode;
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

      // remove the reduced states
      self.state_stack.pop_n(output.reduced);

      // try to push next state to state stack, the next state may be None
      let next_state_exists = self
        .state_stack
        .current()
        .get_next_by_reduced_grammar(&output.nt_grammar_id)
        // update state stack if next state exists
        .map(|next_id| {
          self
            .state_stack
            .push(states.get(&next_id).unwrap().clone().into())
        })
        .is_some();

      // check if an entry NT is reduced as the last node in the reducing stack
      if self.reducing_stack.len() == 1 && entry_nts.contains(&output.nt_grammar_id) {
        // this parse is done, maybe continuable
        // TODO: set next token back to None?
        // TODO: return next token to continue?
        return TryReduceResult::Done {
          // if no next state, the state stack was not updated
          // and the parsing will not be continuable
          continuable: next_state_exists,
        };
      }

      // missing-next is only allowed if the parsing is done
      // so now if next not exists, we should enter panic mode
      if !next_state_exists {
        return TryReduceResult::EnterPanicMode;
      }

      // else, parsing is not done and next exists, continue next try-reduce
    }
  }
}

pub enum TryReduceResult {
  NeedLex,
  EnterPanicMode,
  Done { continuable: bool },
}
