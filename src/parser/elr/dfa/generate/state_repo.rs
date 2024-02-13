use super::{candidate_repo::CandidateRepo, raw_state::RawState};
use crate::{
  lexer::token::TokenKind,
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
  cache: HashMap<BTreeSet<CandidateId>, StateId>,
}

impl StateRepo {
  pub fn with_entry(entry_candidates: BTreeSet<CandidateId>) -> Self {
    let state_id = StateId(0);
    let entry_state = RawState::new(state_id, entry_candidates);
    let mut states = HashMap::new();
    states.insert(state_id, entry_state);
    Self {
      states,
      cache: HashMap::new(),
    }
  }

  pub fn states(&self) -> &HashMap<StateId, RawState> {
    &self.states
  }

  pub fn calc_all_states<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
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
      .map(|(id, state)| (id.clone(), state.candidates().clone()))
      .collect::<Vec<_>>();

    loop {
      let mut generated = Vec::new();
      input_grammar_ids.iter().for_each(|input_grammar_id| {
        unexpanded.iter().for_each(|(id, current_candidates)| {
          match self.generate_next(current_candidates, input_grammar_id, cs, nt_closures) {
            NextResult::New(next_candidates) => {
              generated.push((next_candidates, id, input_grammar_id))
            }
            NextResult::InCache(next_id) => self
              .states
              .get_mut(id)
              .unwrap()
              .append_next(input_grammar_id.clone(), Some(next_id)),
            NextResult::NoNext => {
              // append None to mark it is already calculated
              self
                .states
                .get_mut(id)
                .unwrap()
                .append_next(input_grammar_id.clone(), None)
            }
          }
        });
      });

      if generated.len() == 0 {
        // done
        break;
      }

      unexpanded = generated
        .into_iter()
        .map(|(next_candidates, from_id, input_grammar_id)| {
          // construct new state
          let id = StateId(self.states.len());
          // TODO: prevent the clone, use ref?
          let state = RawState::new(id, next_candidates.clone());

          // update cache
          self.states.insert(id, state);
          self.cache.insert(next_candidates.clone(), id);

          // update next_map
          self
            .states
            .get_mut(from_id)
            .unwrap()
            .append_next(input_grammar_id.clone(), Some(id));

          // convert to unexpanded
          (id, next_candidates)
        })
        .collect();
    }
  }

  /// Return `None` if no next or next is already generated.
  fn generate_next<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
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
  ) -> NextResult<BTreeSet<CandidateId>> {
    let next_candidates =
      Self::calc_next_candidates(current_candidates, input_grammar_id, cs, nt_closures);

    if next_candidates.len() == 0 {
      // no next state
      return NextResult::NoNext;
    }

    // check cache whether the state already created
    if let Some(id) = self.cache.get(&next_candidates) {
      return NextResult::InCache(id.clone()); // TODO: prevent clone
    }

    // create new
    NextResult::New(next_candidates)
  }

  // TODO: merge with generate_next
  fn calc_next_candidates<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
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
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  >(
    self,
    candidates: &Vec<Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global>>>,
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

enum NextResult<NextType> {
  New(NextType),
  InCache(StateId),
  NoNext,
}
