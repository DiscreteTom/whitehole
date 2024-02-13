use crate::{
  lexer::token::TokenKind,
  parser::{
    elr::grammar::{
      grammar::{Grammar, GrammarId},
      grammar_rule::{GrammarRule, GrammarRuleId},
    },
    traverser::Traverser,
  },
};
use std::{
  collections::{HashMap, HashSet},
  rc::Rc,
};

pub struct GrammarRuleRepo<
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind> + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  // vec is hash-able.
  // if 2 grammar rules have the same nt and rule, they are the same grammar rule.
  cache: HashMap<Vec<GrammarId>, GrammarRuleId>,
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
    Self {
      cache: HashMap::new(),
      grs: Vec::new(),
    }
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

  pub fn get_or_add(
    &mut self,
    nt: Rc<Grammar<TKind, NTKind>>,
    rule: Vec<Rc<Grammar<TKind, NTKind>>>,
    expect: HashSet<usize>,
    traverser: Traverser<TKind, NTKind, ASTData, ErrorType, Global>,
  ) -> &GrammarRule<TKind, NTKind, ASTData, ErrorType, Global> {
    let mut key = vec![nt.id().clone()];
    key.extend(rule.iter().map(|g| g.id()));

    if let Some(id) = self.cache.get(&key) {
      return &self.grs[id.0];
    }

    let id = GrammarRuleId(self.grs.len());
    self
      .grs
      .push(Rc::new(GrammarRule::new(id, nt, rule, expect, traverser)));
    self.cache.insert(key, id);
    &self.grs[id.0]
  }
}
