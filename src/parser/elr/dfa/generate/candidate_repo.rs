use super::raw_candidate::RawCandidate;
use crate::{
  lexer::token::TokenKind,
  parser::elr::{
    dfa::candidate::CandidateId,
    grammar::{
      grammar::GrammarId,
      grammar_rule::{GrammarRule, GrammarRuleId},
    },
  },
};
use std::{
  collections::{hash_map::Entry, HashMap},
  rc::Rc,
};

pub struct CandidateRepo<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  gr_cache: HashMap<GrammarRuleId, HashMap<usize, CandidateId>>,
  // TODO: is this needed? can we just store candidates in caches?
  candidates: HashMap<CandidateId, RawCandidate<TKind, NTKind, ASTData, ErrorType, Global>>,
}

impl<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > CandidateRepo<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn with_initial(
    grs: &Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
  ) -> Self {
    let mut gr_cache = HashMap::new();
    let mut candidates = HashMap::new();
    let digested = 0;

    // use index as the candidate_id
    for (candidate_id, gr) in grs.iter().enumerate() {
      let candidate = RawCandidate::new(candidate_id, gr.clone(), digested);
      candidates.insert(candidate_id, candidate);
      gr_cache.insert(gr.id().clone(), {
        let mut v = HashMap::new();
        v.insert(digested, candidate_id);
        v
      });
    }
    Self {
      gr_cache,
      candidates,
    }
  }

  pub fn get_initial(
    &self,
    gr_id: &GrammarRuleId,
  ) -> &RawCandidate<TKind, NTKind, ASTData, ErrorType, Global> {
    self
      .candidates
      .get(&self.gr_cache.get(gr_id).unwrap().get(&0).unwrap())
      .unwrap()
  }

  pub fn get_or_add_next(
    &mut self,
    current_id: &CandidateId,
    input_grammar_id: &GrammarId,
  ) -> Option<&RawCandidate<TKind, NTKind, ASTData, ErrorType, Global>> {
    let new_candidate_id = self.candidates.len();
    let candidate = self.candidates.get_mut(current_id).unwrap();

    if !candidate
      .current()
      // TODO: can we omit the `*` and compare ref directly?
      // will that compare the pointer value?
      .is_some_and(|g| *g.id() == *input_grammar_id)
    {
      // can't digest more, or grammar mismatch
      return None;
    }

    let digested = candidate.digested() + 1;
    match self
      .gr_cache
      .get_mut(&candidate.gr().id())
      .unwrap()
      .entry(digested)
    {
      // cache hit, just return
      Entry::Occupied(o) => return Some(self.candidates.get(o.get()).unwrap()),
      // else, create new candidate
      Entry::Vacant(v) => v.insert(new_candidate_id),
    };
    candidate.set_next(Some(new_candidate_id));
    let new_candidate = RawCandidate::new(new_candidate_id, candidate.gr().clone(), digested);
    Some(
      self
        .candidates
        .entry(new_candidate_id)
        // the entry must be vacant
        // TODO: is this the best way?
        .or_insert(new_candidate),
    )
  }

  pub fn get(&self, id: &CandidateId) -> &RawCandidate<TKind, NTKind, ASTData, ErrorType, Global> {
    self.candidates.get(id).unwrap()
  }
}
