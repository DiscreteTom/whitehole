use super::ParserBuilderGrammar;
use crate::lexer::token::TokenKind;
use std::rc::Rc;

pub struct TempGrammarRule<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
> {
  nt: NTKind,
  rule: Vec<ParserBuilderGrammar<TKind, NTKind>>,
}

impl<TKind: TokenKind<TKind> + 'static, NTKind: TokenKind<NTKind> + Clone + 'static>
  TempGrammarRule<TKind, NTKind>
{
  pub fn new(nt: NTKind, rule: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>) -> Rc<Self> {
    Rc::new(Self {
      nt,
      rule: rule.into(),
    })
  }

  pub fn nt(&self) -> &NTKind {
    &self.nt
  }
  pub fn rule(&self) -> &Vec<ParserBuilderGrammar<TKind, NTKind>> {
    &self.rule
  }
}

pub fn gr<TKind: TokenKind<TKind> + 'static, NTKind: TokenKind<NTKind> + Clone + 'static>(
  nt: NTKind,
  rule: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>,
) -> Rc<TempGrammarRule<TKind, NTKind>> {
  TempGrammarRule::new(nt, rule)
}
