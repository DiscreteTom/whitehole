use std::collections::HashMap;

use crate::{
  lexer::token::TokenKind,
  parser::elr::{
    dfa::{candidate::CandidateId, state::StateId},
    grammar::grammar::{Grammar, GrammarId},
  },
};

pub struct RawState {
  id: StateId,
  candidates: Vec<CandidateId>,
  next_map: HashMap<GrammarId, Option<StateId>>,
}

impl RawState {
  pub fn new(id: StateId, candidates: Vec<CandidateId>) -> Self {
    RawState {
      id,
      candidates,
      next_map: HashMap::default(),
    }
  }

  pub fn id(&self) -> StateId {
    self.id
  }
  pub fn candidates(&self) -> &Vec<CandidateId> {
    &self.candidates
  }

  pub fn generate_next(&self, input_grammar_id: &GrammarId) {
    // TODO
  }
}
