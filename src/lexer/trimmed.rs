use super::{
  expectation::Expectation,
  output::{LexAllOutput, LexOutput, PeekOutput, ReLexContext, TrimOutput},
  state::LexerState,
  stateless::StatelessLexer,
  token::{Token, TokenKind},
  LexOptions, Lexer,
};

/// The `TrimmedLexer` is always trimmed.
pub struct TrimmedLexer<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  /// This should always be trimmed.
  lexer: Lexer<'buffer, Kind, ActionState, ErrorType>,
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  Into<Lexer<'buffer, Kind, ActionState, ErrorType>>
  for TrimmedLexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn into(self) -> Lexer<'buffer, Kind, ActionState, ErrorType> {
    self.lexer
  }
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  From<Lexer<'buffer, Kind, ActionState, ErrorType>>
  for TrimmedLexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn from(mut lexer: Lexer<'buffer, Kind, ActionState, ErrorType>) -> Self {
    lexer.trim();
    TrimmedLexer { lexer }
  }
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static> Clone
  for TrimmedLexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn clone(&self) -> Self {
    TrimmedLexer {
      lexer: self.lexer.clone(),
    }
  }
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  TrimmedLexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  pub fn stateless(&self) -> &StatelessLexer<Kind, ActionState, ErrorType> {
    self.lexer.stateless()
  }
  pub fn state(&self) -> &LexerState<'buffer> {
    self.lexer.state()
  }
  pub fn action_state(&self) -> &ActionState {
    self.lexer.action_state()
  }
  pub fn action_state_mut(&mut self) -> &mut ActionState {
    self.lexer.action_state_mut()
  }

  pub fn reload<'new_buffer>(
    self,
    buffer: &'new_buffer str,
  ) -> Lexer<'new_buffer, Kind, ActionState, ErrorType> {
    // load a new buffer, the result is not a trimmed lexer
    self.lexer.reload(buffer)
  }

  pub fn clone_with<'new_buffer>(
    &self,
    buffer: &'new_buffer str,
  ) -> Lexer<'new_buffer, Kind, ActionState, ErrorType> {
    // load a new buffer, the result is not a trimmed lexer
    self.lexer.clone_with(buffer)
  }

  pub fn peek(&self) -> PeekOutput<Token<'buffer, Kind, ErrorType>, ActionState> {
    self.lexer.peek()
  }

  pub fn peek_expect<'expect_text>(
    &self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> PeekOutput<Token<'buffer, Kind, ErrorType>, ActionState> {
    self.lexer.peek_expect(expectation)
  }

  /// Apply a function to the inner lexer.
  /// After that the inner lexer will be trimmed.
  pub fn apply<F, R>(&mut self, f: F) -> (R, TrimOutput<Token<'buffer, Kind, ErrorType>>)
  where
    F: FnOnce(&mut Lexer<'buffer, Kind, ActionState, ErrorType>) -> R,
  {
    let res = f(&mut self.lexer);
    let output = self.lexer.trim();
    (res, output)
  }

  /// Similar to [`Lexer::lex`], but the lexer is trimmed after that.
  pub fn lex(
    &mut self,
  ) -> (
    LexOutput<
      Token<'buffer, Kind, ErrorType>,
      ReLexContext<Lexer<'buffer, Kind, ActionState, ErrorType>>,
    >,
    TrimOutput<Token<'buffer, Kind, ErrorType>>,
  ) {
    self.apply(|lexer| lexer.lex())
  }
  /// Similar to [`Lexer::lex_expect`], but the lexer is trimmed after that.
  pub fn lex_expect(
    &mut self,
    expectation: Expectation<'_, Kind>,
  ) -> (
    LexOutput<
      Token<'buffer, Kind, ErrorType>,
      ReLexContext<Lexer<'buffer, Kind, ActionState, ErrorType>>,
    >,
    TrimOutput<Token<'buffer, Kind, ErrorType>>,
  ) {
    self.apply(|lexer| lexer.lex_expect(expectation))
  }
  /// Similar to [`Lexer::lex_with`], but the lexer is trimmed after that.
  pub fn lex_with<'expect_text>(
    &mut self,
    options: impl Into<LexOptions<'expect_text, Kind>>,
  ) -> (
    LexOutput<
      Token<'buffer, Kind, ErrorType>,
      ReLexContext<Lexer<'buffer, Kind, ActionState, ErrorType>>,
    >,
    TrimOutput<Token<'buffer, Kind, ErrorType>>,
  ) {
    self.apply(|lexer| lexer.lex_with(options))
  }
  /// Similar to [`Lexer::lex_all`], but the lexer is trimmed after that.
  pub fn lex_all(
    &mut self,
  ) -> (
    LexAllOutput<Token<'buffer, Kind, ErrorType>>,
    TrimOutput<Token<'buffer, Kind, ErrorType>>,
  ) {
    self.apply(|lexer| lexer.lex_all())
  }
  /// Similar to [`Lexer::take`], but the lexer is trimmed after that.
  pub fn take(
    &mut self,
    n: usize,
    state: Option<ActionState>,
  ) -> (&mut Self, TrimOutput<Token<'buffer, Kind, ErrorType>>) {
    let (_, output) = self.apply(|lexer| {
      lexer.take(n, state);
    });
    (self, output)
  }

  // there is no `trim` for TrimmedLexer
}
