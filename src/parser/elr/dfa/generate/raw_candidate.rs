use super::candidate_repo::CandidateRepo;
use crate::{
  lexer::{
    expectation::Expectation,
    token::{Range, TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::{
    ast::ASTNode,
    elr::{
      dfa::candidate::CandidateId,
      grammar::{
        grammar::{Grammar, GrammarId, GrammarKind},
        grammar_rule::GrammarRule,
      },
    },
  },
};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

pub struct RawCandidate<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
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
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
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

  pub fn id(&self) -> CandidateId {
    self.id
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

  pub fn get_or_generate_next(
    &mut self,
    cs: &mut CandidateRepo<TKind, NTKind, ASTData, ErrorType, Global>,
  ) -> Option<CandidateId> {
    // try to retrieve from cache
    if let Some(cache) = &self.next {
      return cache.clone();
    }

    let next = cs.get_or_add_next(self);
    self.next = match &next {
      Some(next) => Some(Some(next.id)),
      None => Some(None),
    };
    next.map(|c| c.id)
  }
}
