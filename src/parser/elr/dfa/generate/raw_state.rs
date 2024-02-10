use crate::parser::elr::{
  dfa::{candidate::CandidateId, state::StateId},
  grammar::grammar::GrammarId,
};
use std::collections::{BTreeSet, HashMap};

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
}
