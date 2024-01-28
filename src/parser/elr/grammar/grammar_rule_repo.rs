use super::grammar_rule::GrammarRule;
use crate::lexer::token::TokenKind;
use std::collections::HashMap;

pub type GrammarRuleID = usize;

pub struct GrammarRuleRepo<'grammar, TKind: TokenKind, NTKind> {
  map: HashMap<GrammarRuleID, GrammarRule<'grammar, TKind, NTKind>>,
}

impl<'grammar, TKind: TokenKind, NTKind> GrammarRuleRepo<'grammar, TKind, NTKind> {
  pub fn new(grs: Vec<GrammarRule<'grammar, TKind, NTKind>>) -> Self {
    Self {
      map: grs.into_iter().enumerate().collect(),
    }
  }
}
