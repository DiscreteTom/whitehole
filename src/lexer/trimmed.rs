use super::{
  expectation::Expectation,
  options::LexOptions,
  output::{LexAllOutput, LexOutput, ReLexable, TrimOutput},
  state::LexerState,
  stateless::StatelessLexer,
  token::{Token, TokenKind},
  Lexer,
};

/// The `TrimmedLexer` is always trimmed.
pub struct TrimmedLexer<'text, Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  /// This should always be trimmed.
  lexer: Lexer<'text, Kind, ActionState, ErrorType>,
}

impl<'text, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  Into<Lexer<'text, Kind, ActionState, ErrorType>>
  for TrimmedLexer<'text, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn into(self) -> Lexer<'text, Kind, ActionState, ErrorType> {
    self.lexer
  }
}

impl<'text, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  From<Lexer<'text, Kind, ActionState, ErrorType>>
  for TrimmedLexer<'text, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn from(mut lexer: Lexer<'text, Kind, ActionState, ErrorType>) -> Self {
    lexer.trim();
    TrimmedLexer { lexer }
  }
}

impl<'text, Kind: 'static, ActionState: 'static, ErrorType: 'static> Clone
  for TrimmedLexer<'text, Kind, ActionState, ErrorType>
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

impl<'text, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  TrimmedLexer<'text, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  // we shouldn't expose the inner lexer
  // so we have to define these proxy methods
  pub fn stateless(&self) -> &StatelessLexer<Kind, ActionState, ErrorType> {
    self.lexer.stateless()
  }
  pub fn state(&self) -> &LexerState<'text> {
    self.lexer.state()
  }
  pub fn action_state(&self) -> &ActionState {
    &self.lexer.action_state
  }
  pub fn action_state_mut(&mut self) -> &mut ActionState {
    &mut self.lexer.action_state
  }

  pub fn reload<'new_text>(
    self,
    text: &'new_text str,
  ) -> Lexer<'new_text, Kind, ActionState, ErrorType> {
    // load a new text, the result is not a trimmed lexer
    self.lexer.reload(text)
  }

  pub fn clone_with<'new_text>(
    &self,
    text: &'new_text str,
  ) -> Lexer<'new_text, Kind, ActionState, ErrorType> {
    // load a new text, the result is not a trimmed lexer
    self.lexer.clone_with(text)
  }

  pub fn peek(
    &self,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  ) {
    self.lexer.peek()
  }

  pub fn peek_expect<'expect_text>(
    &self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  ) {
    self.lexer.peek_expect(expectation)
  }

  pub fn peek_fork(
    &self,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  ) {
    self.lexer.peek_fork()
  }

  pub fn peek_with(
    &self,
    options: LexOptions<'_, Kind>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  ) {
    self.lexer.peek_with(options)
  }

  /// Apply a function to the inner lexer.
  /// After that the inner lexer will be trimmed.
  pub fn apply<F, R>(&mut self, f: F) -> (R, TrimOutput<Token<'text, Kind, ErrorType>>)
  where
    F: FnOnce(&mut Lexer<'text, Kind, ActionState, ErrorType>) -> R,
  {
    let res = f(&mut self.lexer);
    let output = self.lexer.trim();
    (res, output)
  }

  /// Similar to [`Lexer::lex`], but the lexer is trimmed after that.
  pub fn lex(
    &mut self,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<Lexer<'text, Kind, ActionState, ErrorType>>>,
    TrimOutput<Token<'text, Kind, ErrorType>>,
  ) {
    self.apply(|lexer| lexer.lex())
  }
  /// Similar to [`Lexer::lex_expect`], but the lexer is trimmed after that.
  pub fn lex_expect(
    &mut self,
    expectation: Expectation<'_, Kind>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<Lexer<'text, Kind, ActionState, ErrorType>>>,
    TrimOutput<Token<'text, Kind, ErrorType>>,
  ) {
    self.apply(|lexer| lexer.lex_expect(expectation))
  }
  /// Similar to [`Lexer::lex_fork`], but the lexer is trimmed after that.
  pub fn lex_fork(
    &mut self,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<Lexer<'text, Kind, ActionState, ErrorType>>>,
    TrimOutput<Token<'text, Kind, ErrorType>>,
  ) {
    self.apply(|lexer| lexer.lex_fork())
  }
  /// Similar to [`Lexer::lex_with`], but the lexer is trimmed after that.
  pub fn lex_with<'expect_text>(
    &mut self,
    options: impl Into<LexOptions<'expect_text, Kind>>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<Lexer<'text, Kind, ActionState, ErrorType>>>,
    TrimOutput<Token<'text, Kind, ErrorType>>,
  ) {
    self.apply(|lexer| lexer.lex_with(options))
  }
  /// Similar to [`Lexer::lex_all`], but the lexer is trimmed after that.
  pub fn lex_all(
    &mut self,
  ) -> (
    LexAllOutput<Token<'text, Kind, ErrorType>>,
    TrimOutput<Token<'text, Kind, ErrorType>>,
  ) {
    self.apply(|lexer| lexer.lex_all())
  }
  /// Similar to [`Lexer::take`], but the lexer is trimmed after that.
  pub fn take(
    &mut self,
    n: usize,
    state: Option<ActionState>,
  ) -> (&mut Self, TrimOutput<Token<'text, Kind, ErrorType>>) {
    let (_, output) = self.apply(|lexer| {
      lexer.take(n, state);
    });
    (self, output)
  }

  // there is no `trim` for TrimmedLexer
}
