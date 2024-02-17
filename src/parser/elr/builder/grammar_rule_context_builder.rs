use super::{
  conflict::ConflictKind,
  reduce_context::{Condition, ReduceContext},
  temp_grammar_rule::TempGrammarRule,
  temp_resolver::{ReduceShiftResolverOptions, TempResolvedConflict},
};
use crate::lexer::token::TokenKind;
use std::rc::Rc;

pub struct GrammarRuleContextBuilder<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  pub rejecter:
    Condition<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
  pub resolved_conflicts: Vec<
    TempResolvedConflict<
      TKind,
      NTKind,
      Rc<TempGrammarRule<TKind, NTKind>>,
      Condition<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
    >,
  >,
}

impl<
    TKind: TokenKind<TKind> + 'static,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  > Default
  for GrammarRuleContextBuilder<
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >
{
  fn default() -> Self {
    Self {
      rejecter: Box::new(|_| false),
      resolved_conflicts: Vec::new(),
    }
  }
}

impl<
    'a,
    'buffer,
    TKind: TokenKind<TKind> + 'static,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  >
  GrammarRuleContextBuilder<
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >
{
  pub fn rejecter<F>(mut self, condition: F) -> Self
  where
    F: Fn(
        &ReduceContext<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
      ) -> bool
      + 'static,
  {
    self.rejecter = Box::new(condition);
    self
  }

  pub fn resolve_rs<F>(mut self, gr: Rc<TempGrammarRule<TKind, NTKind>>, f: F) -> Self
  where
    F: FnOnce(
      ReduceShiftResolverOptions<
        TKind,
        NTKind,
        ASTData,
        ErrorType,
        Global,
        LexerActionState,
        LexerErrorType,
      >,
    ) -> ReduceShiftResolverOptions<
      TKind,
      NTKind,
      ASTData,
      ErrorType,
      Global,
      LexerActionState,
      LexerErrorType,
    >,
  {
    let ctx = f(ReduceShiftResolverOptions::default());
    self.resolved_conflicts.push(TempResolvedConflict {
      kind: ConflictKind::ReduceShift,
      another_rule: gr,
      accepter: ctx.accepter,
      condition: ctx.condition,
    });
    self
  }
}
