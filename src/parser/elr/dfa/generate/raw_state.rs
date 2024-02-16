use crate::{
  lexer::token::{TokenKind, TokenKindId},
  parser::elr::{
    dfa::{
      candidate::{Candidate, CandidateId},
      state::{State, StateId},
    },
    grammar::{
      grammar::{Grammar, GrammarId},
      grammar_map::GrammarMap,
    },
  },
};
use std::{
  collections::{BTreeSet, HashMap},
  rc::Rc,
};

pub struct RawState {
  id: StateId,
  // store candidates as Rc because we also need this in [[@StateRepo.cache]].
  candidates: Rc<BTreeSet<CandidateId>>,
  next_map: HashMap<GrammarId, Option<StateId>>,
}

impl RawState {
  pub fn new(id: StateId, candidates: Rc<BTreeSet<CandidateId>>) -> Self {
    RawState {
      id,
      candidates,
      next_map: HashMap::default(),
    }
  }

  pub fn id(&self) -> &StateId {
    &self.id
  }
  pub fn candidates(&self) -> &Rc<BTreeSet<CandidateId>> {
    &self.candidates
  }

  pub fn append_next(&mut self, input_grammar_id: GrammarId, next: Option<StateId>) {
    self.next_map.insert(input_grammar_id, next);
  }

  pub fn into_state<
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
  ) -> State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType> {
    State::new(
      self.id,
      self
        .candidates
        .iter()
        .map(|id| candidates[id.0].clone())
        .collect(),
      self.next_map,
      grammar_map,
    )
  }
}
