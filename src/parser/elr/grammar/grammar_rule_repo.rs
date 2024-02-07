use super::grammar_rule::GrammarRule;
use crate::lexer::token::TokenKind;

pub struct GrammarRuleRepo<
  Kind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  grs: Vec<GrammarRule<Kind, ASTData, ErrorType, Global>>,
}

impl<Kind: TokenKind + Clone, ASTData: 'static, ErrorType: 'static, Global: 'static>
  GrammarRuleRepo<Kind, ASTData, ErrorType, Global>
{
  pub fn new(grs: Vec<GrammarRule<Kind, ASTData, ErrorType, Global>>) -> Self {
    Self { grs }
  }
}
