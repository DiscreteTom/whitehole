use super::grammar::Grammar;
use crate::lexer::token::TokenKind;

pub struct GrammarRule<'grammar, TKind: TokenKind, NTKind> {
  rule: Vec<&'grammar Grammar<TKind, NTKind>>,
  nt: NTKind,
}

impl<'grammar, TKind: TokenKind, NTKind> GrammarRule<'grammar, TKind, NTKind> {
  pub fn new(nt: NTKind, rule: Vec<&'grammar Grammar<TKind, NTKind>>) -> Self {
    Self { rule, nt }
  }
  pub fn nt(&self) -> &NTKind {
    &self.nt
  }
  pub fn rule(&self) -> &Vec<&'grammar Grammar<TKind, NTKind>> {
    &self.rule
  }
}
