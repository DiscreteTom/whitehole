use super::grammar::Grammar;
use crate::{lexer::token::TokenKind, parser::traverser::Traverser};
use std::{collections::HashSet, rc::Rc};

pub type GrammarRuleId = usize;

pub struct GrammarRule<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  id: GrammarRuleId,
  nt: NTKind,
  rule: Vec<Rc<Grammar<TKind, NTKind>>>,
  expect: HashSet<usize>,
  traverser: Traverser<TKind, NTKind, ASTData, ErrorType, Global>,
}

impl<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn new(
    id: GrammarRuleId,
    nt: NTKind,
    rule: Vec<Rc<Grammar<TKind, NTKind>>>,
    expect: HashSet<usize>,
    traverser: Traverser<TKind, NTKind, ASTData, ErrorType, Global>,
  ) -> Self {
    Self {
      id,
      nt,
      rule,
      expect,
      traverser,
    }
  }
  pub fn id(&self) -> GrammarRuleId {
    self.id
  }
  pub fn nt(&self) -> &NTKind {
    &self.nt
  }
  pub fn rule(&self) -> &[Rc<Grammar<TKind, NTKind>>] {
    &self.rule
  }
  pub fn expect(&self) -> &HashSet<usize> {
    &self.expect
  }
  pub fn traverser(&self) -> &Traverser<TKind, NTKind, ASTData, ErrorType, Global> {
    &self.traverser
  }

  pub fn at(&self, index: usize) -> Option<&Rc<Grammar<TKind, NTKind>>> {
    self.rule.get(index)
  }
}
