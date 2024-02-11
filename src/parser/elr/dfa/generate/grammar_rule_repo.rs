use crate::{lexer::token::TokenKind, parser::elr::grammar::grammar_rule::GrammarRule};
use std::rc::Rc;

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
  > GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn new(grs: Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>) -> Self {
    Self { grs }
  }

  pub fn grs(&self) -> &Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>> {
    &self.grs
  }
}
