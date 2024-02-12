use crate::{
  lexer::token::TokenKind,
  parser::{
    elr::grammar::{
      grammar::Grammar,
      grammar_rule::{GrammarRule, GrammarRuleId},
    },
    traverser::Traverser,
  },
};
use std::{collections::HashSet, rc::Rc};

pub struct GrammarRuleRepo<
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind> + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  grs: Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > Default for GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>
{
  fn default() -> Self {
    Self { grs: Vec::new() }
  }
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn grs(&self) -> &Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>> {
    &self.grs
  }

  pub fn push(
    &mut self,
    nt: Rc<Grammar<TKind, NTKind>>,
    rule: Vec<Rc<Grammar<TKind, NTKind>>>,
    expect: HashSet<usize>,
    traverser: Traverser<TKind, NTKind, ASTData, ErrorType, Global>,
  ) {
    let id = GrammarRuleId(self.grs.len());
    self
      .grs
      .push(Rc::new(GrammarRule::new(id, nt, rule, expect, traverser)));
  }
}
