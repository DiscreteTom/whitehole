use super::{candidate_repo::CandidateRepo, raw_candidate::RawCandidate, raw_state::RawState};
use crate::{
  lexer::token::{TokenKind, TokenKindId},
  parser::elr::{
    dfa::{
      candidate::{Candidate, CandidateId},
      state::{State, StateId},
    },
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
  // TODO: can we merge these two so we don't need to store BTreeSet twice?
  states: HashMap<StateId, RawState>,
  cache: HashSet<BTreeSet<CandidateId>>,
}

impl StateRepo {
  pub fn with_entry(entry_candidates: BTreeSet<CandidateId>) -> Self {
    let state_id = StateId(0);
    let entry_state = RawState::new(state_id, entry_candidates);
    let mut states = HashMap::new();
    states.insert(state_id, entry_state);
    Self {
      states,
      cache: HashSet::new(),
    }
  }

  pub fn states(&self) -> &HashMap<StateId, RawState> {
    &self.states
  }

  pub fn calc_all_states<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  >(
    &mut self,
    input_grammar_ids: &HashSet<GrammarId>,
    cs: &mut CandidateRepo<TKind, NTKind, ASTData, ErrorType, Global>,
    // TODO: nt_closures only store grammar rule id?
    nt_closures: &HashMap<
      GrammarId,
      Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
    >,
  ) {
    // store the candidates of each unexpanded state
    let mut unexpanded = self
      .states
      .iter()
      // TODO: prevent the clone
      .map(|(_, state)| state.candidates().clone())
      .collect::<Vec<_>>();
    loop {
      let mut generated = Vec::new();
      input_grammar_ids.iter().for_each(|input_grammar_id| {
        unexpanded.iter().for_each(|current_candidates| {
          if let Some(next_candidates) =
            self.generate_next(current_candidates, input_grammar_id, cs, nt_closures)
          {
            generated.push(next_candidates)
          }
        });
      });

      generated.iter().for_each(|next_candidates| {
        let id = StateId(self.states.len());
        // TODO: prevent the clone, use ref?
        let state = RawState::new(id, next_candidates.clone());
        self.states.insert(id, state);
        self.cache.insert(next_candidates.clone());
      });

      unexpanded = generated;
    }
  }

  /// Return `None` if no next or next is already generated.
  fn generate_next<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  >(
    &self,
    current_candidates: &BTreeSet<CandidateId>,
    input_grammar_id: &GrammarId,
    cs: &mut CandidateRepo<TKind, NTKind, ASTData, ErrorType, Global>,
    // TODO: nt_closures only store grammar rule id?
    nt_closures: &HashMap<
      GrammarId,
      Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
    >,
  ) -> Option<BTreeSet<CandidateId>> {
    let next_candidates =
      Self::calc_next_candidates(current_candidates, input_grammar_id, cs, nt_closures);

    if next_candidates.len() == 0 {
      // no next state
      return None;
    }

    // check cache whether the state already created
    if self.cache.contains(&next_candidates) {
      return None;
    }

    // create new
    Some(next_candidates)
  }

  // TODO: merge with generate_next
  fn calc_next_candidates<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  >(
    current_candidates: &BTreeSet<CandidateId>,
    input_grammar_id: &GrammarId,
    cs: &mut CandidateRepo<TKind, NTKind, ASTData, ErrorType, Global>,
    // TODO: nt_closures only store grammar rule id?
    nt_closures: &HashMap<
      GrammarId,
      Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
    >,
  ) -> BTreeSet<CandidateId> {
    // TODO: optimize code
    let mut nts = HashSet::new();
    // find grammar rules that can accept the input grammar
    let mut next_candidates = current_candidates
      .iter()
      .map(|c_id| {
        cs.get_or_add_next(c_id, input_grammar_id).map(|next| {
          if let Some(next_current) = next.current().and_then(|current| {
            if let GrammarKind::NT(_) = current.kind() {
              Some(current)
            } else {
              None
            }
          }) {
            nts.insert(next_current.id().clone());
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

  pub fn into_states<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  >(
    self,
    candidates: &HashMap<CandidateId, Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global>>>,
  ) -> HashMap<StateId, Rc<State<TKind, NTKind, ASTData, ErrorType, Global>>> {
    self
      .states
      .into_iter()
      .map(|(id, state)| {
        let state = state.into_state(candidates);
        (id, Rc::new(state))
      })
      .collect()
  }
}
