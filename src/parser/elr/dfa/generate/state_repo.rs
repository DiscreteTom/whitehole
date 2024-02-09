use super::raw_state::RawState;
use crate::{
  lexer::token::TokenKind,
  parser::elr::{
    dfa::{
      candidate::CandidateId,
      state::{State, StateId},
    },
    grammar::{grammar::GrammarId, grammar_rule::GrammarRule},
  },
};
use std::{collections::HashMap, rc::Rc};

pub struct StateRepo {
  states: HashMap<StateId, RawState>,
}

impl StateRepo {
  pub fn with_entry(entry_candidates: Vec<CandidateId>) -> Self {
    let state_id = 0;
    let entry_state = RawState::new(state_id, entry_candidates);
    let mut states = HashMap::new();
    states.insert(state_id, entry_state);
    Self { states }
  }

  // TODO: only available when enable feature `generate`?
  // pub fn get_or_add_next(
  //   &mut self,
  //   current: &mut State<TKind, NTKind, ASTData, ErrorType, Global>,
  //   input_grammar_id: &GrammarId,
  //   cs: &mut CandidateRepo<TKind, NTKind, ASTData, ErrorType, Global>,
  // ) -> Option<Rc<State<TKind, NTKind, ASTData, ErrorType, Global>>> {
  //   // find grammar rules that can accept the input grammar
  //   let direct_candidates = current
  //     .candidates_mut()
  //     .iter_mut()
  //     .filter(|candidate| {
  //       // ensure candidate can digest more
  //       // and can accept the next grammar
  //       candidate
  //         .current()
  //         .is_some_and(|g| g.id() == *input_grammar_id)
  //     })
  //     .map(|candidate| candidate.get_or_generate_next(cs))
  //     .filter(|c| c.is_some())
  //     .map(|c| c.unwrap())
  //     .collect::<Vec<Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global>>>>();

  //   None
  // }
}
