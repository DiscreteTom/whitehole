use super::{candidate_repo::CandidateRepo, raw_candidate::RawCandidate, raw_state::RawState};
use crate::{
  lexer::token::{TokenKind, TokenKindId},
  parser::elr::{
    dfa::{candidate::CandidateId, state::StateId},
    grammar::{grammar::GrammarId, grammar_rule::GrammarRule},
  },
};
use std::{
  collections::{HashMap, HashSet},
  hash::Hash,
  rc::Rc,
};

pub struct StateRepo {
  states: HashMap<StateId, RawState>,
  cache: HashMap<SortedCandidateIdVec, StateId>,
}

impl StateRepo {
  pub fn with_entry(entry_candidates: Vec<CandidateId>) -> Self {
    let state_id = 0;
    let entry_state = RawState::new(state_id, entry_candidates);
    let mut states = HashMap::new();
    states.insert(state_id, entry_state);
    Self {
      states,
      cache: HashMap::new(),
    }
  }

  pub fn get_or_add_next<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  >(
    &mut self,
    current: &RawState,
    input_grammar_id: &GrammarId,
    cs: &mut CandidateRepo<TKind, NTKind, ASTData, ErrorType, Global>,
    // TODO: nt_closures only store grammar rule id?
    nt_closures: &HashMap<
      TokenKindId,
      Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
    >,
  ) -> Option<StateId> {
    let next_candidates = Self::get_next_candidates(current, input_grammar_id, cs, nt_closures);

    if next_candidates.0.len() == 0 {
      // no next state
      return None;
    }

    // check cache
    if let Some(cache) = self.cache.get(&next_candidates) {
      return Some(cache.clone());
    }

    // create new
    let id = self.states.len();
    let state = RawState::new(id, next_candidates.clone().0);
    self.states.insert(id, state);
    self.cache.insert(next_candidates, id);
    Some(id)
  }

  fn get_next_candidates<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  >(
    current: &RawState,
    input_grammar_id: &GrammarId,
    cs: &mut CandidateRepo<TKind, NTKind, ASTData, ErrorType, Global>,
    // TODO: nt_closures only store grammar rule id?
    nt_closures: &HashMap<
      TokenKindId,
      Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
    >,
  ) -> SortedCandidateIdVec {
    // TODO: optimize code
    let mut nts = HashSet::new();
    // find grammar rules that can accept the input grammar
    let mut next_candidates = current
      .candidates()
      .iter()
      .map(|c_id| {
        cs.get_or_add_next(c_id, input_grammar_id).map(|next| {
          nts.insert(next.gr().nt().id());
          next.id()
        })
      })
      .filter_map(|c| c) // TODO: is this the best way?
      .collect::<Vec<_>>();

    let mut grs = HashSet::new();
    for nt in nts {
      nt_closures.get(&nt).unwrap().iter().for_each(|gr| {
        grs.insert(gr.id());
      });
    }
    grs
      .iter()
      .for_each(|gr_id| next_candidates.push(cs.get_initial(gr_id).id()));

    SortedCandidateIdVec::sort_new(next_candidates)
  }
}

// TODO: is the derive ok?
#[derive(PartialEq, Eq, Hash, Clone)]
struct SortedCandidateIdVec(pub Vec<CandidateId>);

impl SortedCandidateIdVec {
  pub fn sort_new(mut vec: Vec<CandidateId>) -> Self {
    vec.sort();
    Self(vec)
  }
}
