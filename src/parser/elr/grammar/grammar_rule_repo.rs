use super::grammar_rule::GrammarRule;
use crate::lexer::token::TokenKind;

pub struct GrammarRuleRepo<
  TKind: TokenKind,
  NTKind: TokenKind,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  grs: Vec<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>,
}

impl<TKind: TokenKind, NTKind: TokenKind, ASTData: 'static, ErrorType: 'static, Global: 'static>
  GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn new(grs: Vec<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>) -> Self {
    Self { grs }
  }
}
