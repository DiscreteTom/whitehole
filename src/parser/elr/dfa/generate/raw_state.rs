use crate::{
  lexer::token::TokenKind,
  parser::elr::{
    dfa::{
      candidate::{Candidate, CandidateId},
      state::{State, StateId},
    },
    grammar::grammar::GrammarId,
  },
};
use std::{
  collections::{BTreeSet, HashMap},
  rc::Rc,
};

pub struct RawState {
  id: StateId,
  candidates: BTreeSet<CandidateId>,
  next_map: HashMap<GrammarId, Option<StateId>>,
}

impl RawState {
  pub fn new(id: StateId, candidates: BTreeSet<CandidateId>) -> Self {
    RawState {
      id,
      candidates,
      next_map: HashMap::default(),
    }
  }

  pub fn id(&self) -> &StateId {
    &self.id
  }
  pub fn candidates(&self) -> &BTreeSet<CandidateId> {
    &self.candidates
  }

  pub fn append_next(&mut self, input_grammar_id: GrammarId, next: Option<StateId>) {
    self.next_map.insert(input_grammar_id, next);
  }

  pub fn into_state<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  >(
    self,
    candidates: &Vec<Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global>>>,
  ) -> State<TKind, NTKind, ASTData, ErrorType, Global> {
    State::new(
      self.id,
      self
        .candidates
        .iter()
        .map(|id| candidates[id.0].clone())
        .collect(),
      self.next_map,
    )
  }
}
