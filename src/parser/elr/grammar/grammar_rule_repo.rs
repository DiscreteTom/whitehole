use super::grammar_rule::GrammarRule;
use crate::lexer::token::TokenKind;

pub struct GrammarRuleRepo<Kind: TokenKind> {
  grs: Vec<GrammarRule<Kind>>,
}

impl<Kind: TokenKind> GrammarRuleRepo<Kind> {
  pub fn new(grs: Vec<GrammarRule<Kind>>) -> Self {
    Self { grs }
  }
}
