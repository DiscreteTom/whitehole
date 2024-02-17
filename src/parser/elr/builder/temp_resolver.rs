use super::{conflict::ConflictKind, reduce_context::Condition, ParserBuilderGrammar};
use crate::lexer::token::TokenKind;

pub enum TempResolvedConflictNext<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
> {
  Any,
  Some(Vec<ParserBuilderGrammar<TKind, NTKind>>),
}

pub struct TempResolvedConflictCondition<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
> {
  pub next: TempResolvedConflictNext<TKind, NTKind>,
  pub eof: bool,
}

pub struct ReduceReduceResolverOptions<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  pub accepter:
    Condition<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
  pub condition: TempResolvedConflictCondition<TKind, NTKind>,
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
  for ReduceReduceResolverOptions<
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
      accepter: Box::new(|_| true),
      condition: TempResolvedConflictCondition {
        next: TempResolvedConflictNext::Some(Vec::new()),
        eof: false,
      },
    }
  }
}

impl<
    TKind: TokenKind<TKind> + 'static,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  >
  ReduceReduceResolverOptions<
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >
{
  pub fn accept(mut self, accept: bool) -> Self {
    self.accepter = Box::new(move |_| accept);
    self
  }

  pub fn accepter(
    mut self,
    accepter: Condition<
      TKind,
      NTKind,
      ASTData,
      ErrorType,
      Global,
      LexerActionState,
      LexerErrorType,
    >,
  ) -> Self {
    self.accepter = accepter;
    self
  }

  pub fn on_any_next(mut self) -> Self {
    self.condition.next = TempResolvedConflictNext::Any;
    self
  }

  pub fn handle_eof(mut self) -> Self {
    self.condition.eof = true;
    self
  }

  pub fn on_next(mut self, next: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>) -> Self {
    self.condition.next = TempResolvedConflictNext::Some(next.into());
    self
  }

  pub fn accept_on_any_next(self, accept: bool) -> Self {
    self.on_any_next().accept(accept)
  }

  pub fn accept_on_eof(self, accept: bool) -> Self {
    self.handle_eof().accept(accept)
  }

  pub fn accept_on_next(
    self,
    accept: bool,
    next: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>,
  ) -> Self {
    self.on_next(next).accept(accept)
  }
}

pub struct ReduceShiftResolverOptions<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  pub accepter:
    Condition<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
  pub condition: TempResolvedConflictCondition<TKind, NTKind>,
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
  for ReduceShiftResolverOptions<
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
      accepter: Box::new(|_| true),
      condition: TempResolvedConflictCondition {
        next: TempResolvedConflictNext::Some(Vec::new()),
        eof: false,
      },
    }
  }
}

impl<
    TKind: TokenKind<TKind> + 'static,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  >
  ReduceShiftResolverOptions<
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >
{
  // TODO: optimize code, reduce duplicate code
  pub fn accept(mut self, accept: bool) -> Self {
    self.accepter = Box::new(move |_| accept);
    self
  }

  pub fn accepter(
    mut self,
    accepter: Condition<
      TKind,
      NTKind,
      ASTData,
      ErrorType,
      Global,
      LexerActionState,
      LexerErrorType,
    >,
  ) -> Self {
    self.accepter = accepter;
    self
  }

  pub fn on_any_next(mut self) -> Self {
    self.condition.next = TempResolvedConflictNext::Any;
    self
  }

  pub fn on_next(mut self, next: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>) -> Self {
    self.condition.next = TempResolvedConflictNext::Some(next.into());
    self
  }

  pub fn accept_on_any_next(self, accept: bool) -> Self {
    self.on_any_next().accept(accept)
  }

  pub fn accept_on_next(
    self,
    accept: bool,
    next: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>,
  ) -> Self {
    self.on_next(next).accept(accept)
  }

  // R-S conflict doesn't need to handle EOF
}

pub struct TempResolvedConflict<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  GrammarRuleType,
  AccepterType,
> {
  pub kind: ConflictKind,
  /// If this is a R-S conflict, this rule is a shifter rule. If this is a R-R conflict, this rule is a reducer rule.
  pub another_rule: GrammarRuleType,
  pub accepter: AccepterType,
  pub condition: TempResolvedConflictCondition<TKind, NTKind>,
}
