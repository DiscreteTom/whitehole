use super::reduce_context::{Condition, ReduceContext};
use crate::lexer::token::TokenKind;

pub struct GrammarRuleContextBuilder<
  'a,
  'buffer,
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  rejecter: Condition<
    'a,
    'buffer,
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >,
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
  > Default
  for GrammarRuleContextBuilder<
    'a,
    'buffer,
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
    'a,
    'buffer,
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
        ReduceContext<
          'a,
          'buffer,
          TKind,
          NTKind,
          ASTData,
          ErrorType,
          Global,
          LexerActionState,
          LexerErrorType,
        >,
      ) -> bool
      + 'static,
  {
    self.rejecter = Box::new(condition);
  }
}
