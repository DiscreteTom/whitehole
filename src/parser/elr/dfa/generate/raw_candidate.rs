use crate::{
  lexer::token::TokenKind,
  parser::elr::{
    dfa::candidate::{Candidate, CandidateId},
    grammar::{grammar::Grammar, grammar_rule::GrammarRule},
  },
};
use std::rc::Rc;

pub struct RawCandidate<
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind> + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  id: CandidateId,
  gr: Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>,
  digested: usize,
  /// `None` if not calculated yet, `Some(None)` if no next, `Some(Some(id))` if has next.
  next: Option<Option<CandidateId>>,
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > RawCandidate<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn new(
    id: CandidateId,
    gr: Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>,
    digested: usize,
  ) -> Self {
    Self {
      id,
      gr,
      digested,
      next: None,
    }
  }

  pub fn id(&self) -> &CandidateId {
    &self.id
  }
  pub fn gr(&self) -> &Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>> {
    &self.gr
  }
  pub fn digested(&self) -> usize {
    self.digested
  }

  pub fn current(&self) -> Option<&Rc<Grammar<TKind, NTKind>>> {
    self.gr.rule().get(self.digested)
  }
  pub fn can_digest_more(&self) -> bool {
    self.digested < self.gr.rule().len() - 1
  }

  pub fn set_next(&mut self, next: Option<CandidateId>) {
    self.next = Some(next);
  }

  pub fn into_candidate(self) -> Candidate<TKind, NTKind, ASTData, ErrorType, Global> {
    Candidate::new(self.id, self.gr, self.digested)
  }
}
