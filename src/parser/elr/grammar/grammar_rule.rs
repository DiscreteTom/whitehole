use super::grammar::Grammar;
use crate::{
  lexer::token::TokenKind,
  parser::{elr::builder::reduce_context::Condition, traverser::Traverser},
};
use std::{collections::HashSet, rc::Rc};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct GrammarRuleId(pub usize);

pub struct GrammarRule<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  id: GrammarRuleId,
  // the NT should be a `Grammar` instead of an `NTKind`
  // because we need the grammar to get next state when `try_reduce`
  nt: Rc<Grammar<TKind, NTKind>>,
  rule: Vec<Rc<Grammar<TKind, NTKind>>>,
  expect: HashSet<usize>,
  rejecter: Condition<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
  traverser: Option<Traverser<TKind, NTKind, ASTData, ErrorType, Global>>,
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  > GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  pub fn new(
    id: GrammarRuleId,
    nt: Rc<Grammar<TKind, NTKind>>,
    rule: Vec<Rc<Grammar<TKind, NTKind>>>,
    expect: HashSet<usize>,
    rejecter: Condition<
      TKind,
      NTKind,
      ASTData,
      ErrorType,
      Global,
      LexerActionState,
      LexerErrorType,
    >,
    traverser: Option<Traverser<TKind, NTKind, ASTData, ErrorType, Global>>,
  ) -> Self {
    Self {
      id,
      nt,
      rule,
      expect,
      rejecter,
      traverser,
    }
  }
  pub fn id(&self) -> &GrammarRuleId {
    &self.id
  }
  pub fn nt(&self) -> &Rc<Grammar<TKind, NTKind>> {
    &self.nt
  }
  pub fn rule(&self) -> &[Rc<Grammar<TKind, NTKind>>] {
    &self.rule
  }
  pub fn expect(&self) -> &HashSet<usize> {
    &self.expect
  }
  pub fn rejecter(
    &self,
  ) -> &Condition<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType> {
    &self.rejecter
  }
  pub fn traverser(&self) -> &Option<Traverser<TKind, NTKind, ASTData, ErrorType, Global>> {
    &self.traverser
  }
}
