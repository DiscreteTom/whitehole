use super::{
  candidate::Candidate,
  state::{State, StateId},
};
use crate::{
  lexer::token::TokenKind,
  parser::elr::grammar::{grammar::GrammarId, grammar_rule::GrammarRule},
};
use std::{collections::HashMap, rc::Rc};

pub struct StateRepo<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  states: HashMap<StateId, Rc<State<TKind, NTKind, ASTData, ErrorType, Global>>>,
}

impl<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > StateRepo<TKind, NTKind, ASTData, ErrorType, Global>
{
  // TODO: only available when enable feature `generate`?
  pub fn with_entry(
    entry_candidates: Vec<Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global>>>,
  ) -> Self {
    let state_id = 0;
    let entry_state = Rc::new(State::new(state_id, entry_candidates));
    let mut states = HashMap::new();
    states.insert(state_id, entry_state);
    Self { states }
  }

  // TODO: only available when enable feature `generate`?
  // pub fn get_next(
  //   &mut self,
  //   current: &State<TKind, NTKind, ASTData, ErrorType, Global>,
  //   input_grammar_id: &GrammarId,
  // ) -> Option<Rc<State<TKind, NTKind, ASTData, ErrorType, Global>>> {
  //   // find grammar rules that can accept the input grammar
  //   let direct_candidates = current
  //     .candidates()
  //     .iter()
  //     .filter(|gr| {
  //       gr.at(current.digested())
  //         .is_some_and(|g| g.id() == *input_grammar_id)
  //     })
  //     .map(|gr| gr.clone())
  //     .collect::<Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>>();

  //   None
  // }
}
