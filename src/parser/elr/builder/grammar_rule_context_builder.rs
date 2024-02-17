use super::{
  reduce_context::{Condition, ReduceContext},
  temp_grammar_rule::TempGrammarRule,
  temp_resolver::ReduceShiftResolverOptions,
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
  pub fn rejecter<F>(&mut self, condition: F)
  where
    F: Fn(
        &ReduceContext<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
      ) -> bool
      + 'static,
  {
    self.rejecter = Box::new(condition);
  }

  pub fn resolve_rs<F>(&mut self, gr: Rc<TempGrammarRule<TKind, NTKind>>, f: F)
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
  }
}
