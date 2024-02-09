use super::{candidate_repo::CandidateRepo, raw_candidate::RawCandidate, raw_state::RawState};
use crate::{
  lexer::token::{TokenKind, TokenKindId},
  parser::elr::{
    dfa::{candidate::CandidateId, state::StateId},
    grammar::{
      grammar::{GrammarId, GrammarKind},
      grammar_rule::GrammarRule,
    },
  },
};
use std::{
  collections::{BTreeSet, HashMap, HashSet},
  rc::Rc,
};

pub struct StateRepo {
  states: HashMap<StateId, RawState>,
  // BTreeSet will store elements in a sorted order and can be hashed
  cache: HashMap<BTreeSet<CandidateId>, StateId>,
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

    if next_candidates.len() == 0 {
      // no next state
      return None;
    }

    // check cache whether the state already created
    // TODO: check another cache by current and input grammar id?
    if let Some(cache) = self.cache.get(&next_candidates) {
      return Some(cache.clone());
    }

    // create new
    let id = self.states.len();
    let state = RawState::new(
      id,
      next_candidates
        .iter()
        .map(|c| c.clone())
        .collect::<Vec<_>>(),
    );
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
  ) -> BTreeSet<CandidateId> {
    // TODO: optimize code
    let mut nts = HashSet::new();
    // find grammar rules that can accept the input grammar
    let mut next_candidates = current
      .candidates()
      .iter()
      .map(|c_id| {
        cs.get_or_add_next(c_id, input_grammar_id).map(|next| {
          if let Some(nt) = next.current().and_then(|current| {
            if let GrammarKind::NT(nt) = current.kind() {
              Some(nt)
            } else {
              None
            }
          }) {
            nts.insert(nt.id());
          }
          next.id().clone()
        })
      })
      .filter_map(|c| c) // TODO: is this the best way?
      .collect::<BTreeSet<_>>();

    let mut grs = HashSet::new();
    for nt in nts {
      nt_closures.get(&nt).unwrap().iter().for_each(|gr| {
        grs.insert(gr.id());
      });
    }
    grs.iter().for_each(|gr_id| {
      next_candidates.insert(cs.get_initial(gr_id).id().clone());
    });

    next_candidates
  }
}
