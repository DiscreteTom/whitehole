use super::grammar::Grammar;
use crate::{
  lexer::token::TokenKind,
  parser::{
    elr::builder::{reduce_context::Condition, resolver::ResolvedConflict},
    traverser::Traverser,
  },
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
  // id, nt and rule is not mutable
  id: GrammarRuleId,
  // the NT should be a `Grammar` instead of an `NTKind`
  // because we need the grammar to get next state when `try_reduce`
  nt: Rc<Grammar<TKind, NTKind>>,
  rule: Vec<Rc<Grammar<TKind, NTKind>>>,

  // these are mutable
  pub expect: HashSet<usize>,
  pub resolved_conflicts: Vec<
    ResolvedConflict<
      GrammarRuleId,
      Condition<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
    >,
  >,
  pub rejecter:
    Condition<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
  pub traverser: Option<Traverser<TKind, NTKind, ASTData, ErrorType, Global>>,
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
  ) -> Self {
    Self {
      id,
      nt,
      rule,
      expect: HashSet::new(),
      resolved_conflicts: Vec::new(),
      rejecter: Box::new(|_| false),
      traverser: None,
    }
  }

  pub fn id(&self) -> &GrammarRuleId {
    &self.id
  }
  pub fn nt(&self) -> &Rc<Grammar<TKind, NTKind>> {
    &self.nt
  }
  pub fn rule(&self) -> &Vec<Rc<Grammar<TKind, NTKind>>> {
    &self.rule
  }
}
