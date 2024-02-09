use super::grammar_rule::GrammarRule;
use crate::lexer::token::TokenKind;
use std::rc::Rc;

pub struct GrammarRuleRepo<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  grs: Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
}

impl<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn new(grs: Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>) -> Self {
    Self { grs }
  }

  pub fn grs(&self) -> &Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>> {
    &self.grs
  }
}
