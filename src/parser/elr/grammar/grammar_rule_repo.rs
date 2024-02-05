use super::grammar_rule::GrammarRule;
use crate::lexer::token::TokenKind;

pub struct GrammarRuleRepo<TKind: TokenKind, NTKind: TokenKind> {
  grs: Vec<GrammarRule<TKind, NTKind>>,
}

impl<TKind: TokenKind, NTKind: TokenKind> GrammarRuleRepo<TKind, NTKind> {
  pub fn new(grs: Vec<GrammarRule<TKind, NTKind>>) -> Self {
    Self { grs }
  }
}
