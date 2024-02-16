use super::{candidate_repo::CandidateRepo, raw_state::RawState};
use crate::{
  lexer::token::{TokenKind, TokenKindId},
  parser::elr::{
    dfa::{
      candidate::{Candidate, CandidateId},
      state::{State, StateId},
    },
    grammar::{
      grammar::{Grammar, GrammarId, GrammarKind},
      grammar_map::GrammarMap,
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
  // BTreeSet is ordered and hash-able, HashSet is not.
  // if 2 states have the same sorted candidates, they are the same state.
  // we use Rc to prevent clone. [[StateRepo.cache]]
  cache: HashMap<Rc<BTreeSet<CandidateId>>, StateId>,
}

impl StateRepo {
  pub fn with_entry(entry_candidates: BTreeSet<CandidateId>) -> Self {
    let entry_candidates = Rc::new(entry_candidates);
    let state_id = StateId(0);
    let entry_state = RawState::new(state_id, entry_candidates.clone());
    let mut states = HashMap::new();
    states.insert(state_id, entry_state);
    let mut cache = HashMap::new();
    cache.insert(entry_candidates, state_id);
    Self { states, cache }
  }

  pub fn states(&self) -> &HashMap<StateId, RawState> {
    &self.states
  }

  pub fn calc_all_states<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  >(
    &mut self,
    input_grammar_ids: &HashSet<GrammarId>,
    cs: &mut CandidateRepo<
      TKind,
      NTKind,
      ASTData,
      ErrorType,
      Global,
      LexerActionState,
      LexerErrorType,
    >,
    nt_closures: &HashMap<
      GrammarId,
      Vec<
        Rc<
          GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
        >,
      >,
    >,
  ) {
    // store the candidates of each unexpanded state
    let mut unexpanded = self
      .states
      .iter()
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
          let next_candidates = Rc::new(next_candidates);
          // construct new state
          let id = StateId(self.states.len());
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

  fn generate_next<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  >(
    &self,
    current_candidates: &BTreeSet<CandidateId>,
    input_grammar_id: &GrammarId,
    cs: &mut CandidateRepo<
      TKind,
      NTKind,
      ASTData,
      ErrorType,
      Global,
      LexerActionState,
      LexerErrorType,
    >,
    nt_closures: &HashMap<
      GrammarId,
      Vec<
        Rc<
          GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
        >,
      >,
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
      return NextResult::InCache(id.clone());
    }

    // create new
    NextResult::New(next_candidates)
  }

  fn calc_next_candidates<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  >(
    current_candidates: &BTreeSet<CandidateId>,
    input_grammar_id: &GrammarId,
    cs: &mut CandidateRepo<
      TKind,
      NTKind,
      ASTData,
      ErrorType,
      Global,
      LexerActionState,
      LexerErrorType,
    >,
    nt_closures: &HashMap<
      GrammarId,
      Vec<
        Rc<
          GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
        >,
      >,
    >,
  ) -> BTreeSet<CandidateId> {
    let mut to_be_expanded_nts = HashSet::new();

    // find candidates that can accept the input grammar and use their next as next state's direct candidates
    let mut next_candidates = current_candidates
      .iter()
      .map(|c_id| {
        cs.get_or_add_next(c_id, input_grammar_id).map(|next| {
          // collect NTs that can be expanded
          if let Some(next_current) = next.current().and_then(|current| {
            if let GrammarKind::NT(_) = current.kind() {
              Some(current)
            } else {
              None
            }
          }) {
            to_be_expanded_nts.insert(next_current.id().clone());
          }

          // return next candidate id
          next.id().clone()
        })
      })
      .flatten() // filter out None
      .collect::<BTreeSet<_>>();

    // expand NTs to get indirect candidates for the next state
    for nt in to_be_expanded_nts {
      nt_closures.get(&nt).unwrap().iter().for_each(|gr| {
        next_candidates.insert(cs.get_initial(gr.id()).id().clone());
      });
    }

    next_candidates
  }

  pub fn into_states<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  >(
    self,
    candidates: &Vec<
      Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    >,
    grammar_map: Rc<GrammarMap<TKind, NTKind>>,
  ) -> HashMap<
    StateId,
    Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  > {
    self
      .states
      .into_iter()
      .map(|(id, state)| {
        let state = state.into_state(candidates, grammar_map.clone());
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
