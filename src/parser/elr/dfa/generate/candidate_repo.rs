use super::raw_candidate::RawCandidate;
use crate::{
  lexer::token::TokenKind,
  parser::elr::{
    dfa::candidate::CandidateId,
    grammar::grammar_rule::{GrammarRule, GrammarRuleId},
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
      gr_cache.insert(gr.id(), {
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

  /// Return `None` if the candidate can't digest more.
  pub fn get_or_add_next(
    &mut self,
    c: &RawCandidate<TKind, NTKind, ASTData, ErrorType, Global>,
  ) -> Option<&RawCandidate<TKind, NTKind, ASTData, ErrorType, Global>> {
    if !c.can_digest_more() {
      return None;
    }

    let gr_id = c.gr().id();
    let digested = c.digested() + 1;
    match self.gr_cache.get_mut(&gr_id).unwrap().entry(digested) {
      Entry::Occupied(o) => Some(self.candidates.get(o.get()).unwrap().clone()),
      Entry::Vacant(v) => {
        let id = self.candidates.len();
        let res = RawCandidate::new(id, c.gr().clone(), digested);
        v.insert(id);
        Some(
          self
            .candidates
            .entry(id)
            // the entry must be vacant, so we can just insert
            .or_insert(res),
        )
      }
    }
  }

  pub fn get(&self, id: &CandidateId) -> &RawCandidate<TKind, NTKind, ASTData, ErrorType, Global> {
    self.candidates.get(id).unwrap()
  }
}
