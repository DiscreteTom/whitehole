use super::raw_candidate::RawCandidate;
use crate::{
  lexer::token::TokenKind,
  parser::elr::{
    dfa::candidate::{Candidate, CandidateId},
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
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind> + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  gr_cache: HashMap<GrammarRuleId, HashMap</* digested */ usize, CandidateId>>,
  // TODO: is this needed? can we just store candidates in caches?
  candidates: Vec<RawCandidate<TKind, NTKind, ASTData, ErrorType, Global>>,
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > CandidateRepo<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn with_initial(
    grs: &Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
  ) -> Self {
    let mut gr_cache = HashMap::new();
    let mut candidates = Vec::new();
    let digested = 0;

    // use index as the candidate_id
    for (i, gr) in grs.iter().enumerate() {
      let candidate_id = CandidateId(i);
      let candidate = RawCandidate::new(candidate_id, gr.clone(), digested);
      candidates.push(candidate);
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
    &self.candidates[self.gr_cache.get(gr_id).unwrap().get(&0).unwrap().0]
  }

  pub fn get_or_add_next(
    &mut self,
    current_id: &CandidateId,
    input_grammar_id: &GrammarId,
  ) -> Option<&RawCandidate<TKind, NTKind, ASTData, ErrorType, Global>> {
    let new_candidate_id = CandidateId(self.candidates.len());
    let candidate = &mut self.candidates[current_id.0];

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
      Entry::Occupied(o) => return Some(&self.candidates[o.get().0]),
      // else, create new candidate
      Entry::Vacant(v) => v.insert(new_candidate_id),
    };
    candidate.set_next(Some(new_candidate_id));
    let new_candidate = RawCandidate::new(new_candidate_id, candidate.gr().clone(), digested);
    self.candidates.push(new_candidate);
    Some(&self.candidates[new_candidate_id.0])
  }

  pub fn get(&self, id: &CandidateId) -> &RawCandidate<TKind, NTKind, ASTData, ErrorType, Global> {
    &self.candidates[id.0]
  }

  pub fn into_candidates(self) -> Vec<Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global>>> {
    self
      .candidates
      .into_iter()
      .map(|c| Rc::new(c.into_candidate()))
      .collect()
  }
}
