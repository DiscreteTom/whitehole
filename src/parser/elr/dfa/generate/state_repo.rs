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
  rc::Rc,
};

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
  ) -> Option<&RawState> {
    let next_candidates = Self::get_next_candidates(current, input_grammar_id, cs, nt_closures);

    None
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
  ) -> Vec<CandidateId> {
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

    next_candidates.sort();
    next_candidates
  }
}
