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
pub struct TrimmedLexer<'text, Kind, ActionState, ErrorType> {
  /// This should always be trimmed.
  lexer: Lexer<'text, Kind, ActionState, ErrorType>,
}

impl<'text, Kind, ActionState, ErrorType> Into<Lexer<'text, Kind, ActionState, ErrorType>>
  for TrimmedLexer<'text, Kind, ActionState, ErrorType>
{
  fn into(self) -> Lexer<'text, Kind, ActionState, ErrorType> {
    self.lexer
  }
}

impl<'text, Kind, ActionState, ErrorType> From<Lexer<'text, Kind, ActionState, ErrorType>>
  for TrimmedLexer<'text, Kind, ActionState, ErrorType>
{
  fn from(mut lexer: Lexer<'text, Kind, ActionState, ErrorType>) -> Self {
    lexer.trim();
    TrimmedLexer { lexer }
  }
}

impl<'text, Kind, ActionState, ErrorType> Clone
  for TrimmedLexer<'text, Kind, ActionState, ErrorType>
where
  ActionState: Clone,
{
  fn clone(&self) -> Self {
    TrimmedLexer {
      lexer: self.lexer.clone(),
    }
  }
}

impl<'text, Kind, ActionState, ErrorType> TrimmedLexer<'text, Kind, ActionState, ErrorType> {
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

  /// Similar to [`Lexer::reload_with`], but the lexer is trimmed after that.
  pub fn reload_with<'new_text>(
    self,
    action_state: ActionState,
    text: &'new_text str,
  ) -> TrimmedLexer<'new_text, Kind, ActionState, ErrorType> {
    self.lexer.reload_with(action_state, text).into()
  }

  /// Similar to [`Lexer::reload`], but the lexer is trimmed after that.
  pub fn reload<'new_text>(
    self,
    text: &'new_text str,
  ) -> TrimmedLexer<'new_text, Kind, ActionState, ErrorType>
  where
    ActionState: Default,
  {
    self.lexer.reload(text).into()
  }

  /// Similar to [`Lexer::clone_with`], but the lexer is trimmed after that.
  pub fn clone_with<'new_text>(
    &self,
    action_state: ActionState,
    text: &'new_text str,
  ) -> TrimmedLexer<'new_text, Kind, ActionState, ErrorType> {
    self.lexer.clone_with(action_state, text).into()
  }

  /// Similar to [`Lexer::clone_with_default_action_state`], but the lexer is trimmed after that.
  pub fn clone_with_default_action_state<'new_text>(
    &self,
    text: &'new_text str,
  ) -> TrimmedLexer<'new_text, Kind, ActionState, ErrorType>
  where
    ActionState: Default,
  {
    self.lexer.clone_with_default_action_state(text).into()
  }

  /// Same as [`Lexer::peek`].
  pub fn peek(
    &self,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.lexer.peek()
  }

  /// Same as [`Lexer::peek_expect`].
  pub fn peek_expect<'expect_text>(
    &self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.lexer.peek_expect(expectation)
  }

  /// Same as [`Lexer::peek_fork`].
  pub fn peek_fork(
    &self,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.lexer.peek_fork()
  }

  /// Same as [`Lexer::peek_with`].
  pub fn peek_with(
    &self,
    options: LexOptions<'_, Kind>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
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
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.apply(|lexer| lexer.lex())
  }
  /// Similar to [`Lexer::lex_expect`], but the lexer is trimmed after that.
  pub fn lex_expect(
    &mut self,
    expectation: Expectation<'_, Kind>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<Lexer<'text, Kind, ActionState, ErrorType>>>,
    TrimOutput<Token<'text, Kind, ErrorType>>,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.apply(|lexer| lexer.lex_expect(expectation))
  }
  /// Similar to [`Lexer::lex_fork`], but the lexer is trimmed after that.
  pub fn lex_fork(
    &mut self,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<Lexer<'text, Kind, ActionState, ErrorType>>>,
    TrimOutput<Token<'text, Kind, ErrorType>>,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.apply(|lexer| lexer.lex_fork())
  }
  /// Similar to [`Lexer::lex_with`], but the lexer is trimmed after that.
  pub fn lex_with<'expect_text>(
    &mut self,
    options: impl Into<LexOptions<'expect_text, Kind>>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<Lexer<'text, Kind, ActionState, ErrorType>>>,
    TrimOutput<Token<'text, Kind, ErrorType>>,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.apply(|lexer| lexer.lex_with(options))
  }
  /// Similar to [`Lexer::lex_all`], but the lexer is trimmed after that.
  pub fn lex_all(
    &mut self,
  ) -> (
    LexAllOutput<Token<'text, Kind, ErrorType>>,
    TrimOutput<Token<'text, Kind, ErrorType>>,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.apply(|lexer| lexer.lex_all())
  }
  /// Similar to [`Lexer::take_with`], but the lexer is trimmed after that.
  pub fn take_with(
    &mut self,
    n: usize,
    state: ActionState,
  ) -> (&mut Self, TrimOutput<Token<'text, Kind, ErrorType>>) {
    let (_, output) = self.apply(|lexer| {
      lexer.take_with(n, state);
    });
    (self, output)
  }
  /// Similar to [`Lexer::take`], but the lexer is trimmed after that.
  pub fn take(&mut self, n: usize) -> (&mut Self, TrimOutput<Token<'text, Kind, ErrorType>>)
  where
    ActionState: Default,
  {
    let (_, output) = self.apply(|lexer| {
      lexer.take(n);
    });
    (self, output)
  }

  // there is no `trim` for TrimmedLexer
}
