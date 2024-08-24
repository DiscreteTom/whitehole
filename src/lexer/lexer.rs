use super::{
  fork::{ForkOutputFactory, LexOptionsFork},
  instant::Instant,
  options::{LexOptions, TrimOptions},
  output::{LexOutput, TrimOutput},
  snapshot::Snapshot,
  stateless::{StatelessLexOptions, StatelessLexer, StatelessTrimOptions},
  token::Token,
};
use std::rc::Rc;

/// This is the "stateful" lexer, it manages the [`Instant`] and the `State`.
/// The [`Instant`] is responsible to manage the text and the progress of the lexer.
/// The `State` is provided by you and can be accessed by [`Action`](crate::lexer::action::Action)s
/// to realize stateful lexing.
///
/// If you want a stateless experience, you can use [`StatelessLexer`].
///
/// To create a lexer, you should use [`LexerBuilder`](crate::lexer::LexerBuilder).
/// # Design
/// ## Why there is no `Lexer::errors` to store all the errors?
/// Why the error accumulator is not a field of [`Lexer`]
/// just like [`Lexer::state`],
/// but a field of [`LexOptions`] which needs to be provided every time?
///
/// [`Lexer::state`] is just a value, but the error accumulator is a collection/container.
/// We don't want unnecessary memory allocation, so we won't create the container
/// for users. Users can create their own accumulator and manage its memory allocation.
/// E.g. some users may just want to print the errors, so they don't need any container;
/// some users may want to process errors after each lexing, and clear the container
/// before next lexing to save memory; some users may want to store all errors
/// in a container and process them later.
#[derive(Debug)]
pub struct Lexer<'text, Kind, State = ()> {
  /// You can mutate this directly if needed.
  pub state: State,

  // use Rc so that this is clone-able
  stateless: Rc<StatelessLexer<Kind, State>>,
  instant: Instant<'text>,
}

impl<'text, Kind, State: Clone> Clone for Lexer<'text, Kind, State> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      state: self.state.clone(),
      stateless: self.stateless.clone(),
      instant: self.instant.clone(),
    }
  }
}

impl<'text, Kind, State> Lexer<'text, Kind, State> {
  /// Create a new lexer with the given stateless lexer, state and text.
  /// For most cases you should use [`LexerBuilder`](crate::lexer::LexerBuilder)
  /// to create a lexer.
  #[inline]
  pub const fn new(
    stateless: Rc<StatelessLexer<Kind, State>>,
    state: State,
    text: &'text str,
  ) -> Self {
    Self {
      state,
      stateless,
      instant: Instant::new(text),
    }
  }

  /// Get the stateless lexer.
  #[inline]
  pub const fn stateless(&self) -> &Rc<StatelessLexer<Kind, State>> {
    &self.stateless
  }
  /// Get the lexer state.
  /// You are not allowed to mutate the lexer state directly.
  #[inline]
  pub const fn instant(&self) -> &Instant<'text> {
    &self.instant
  }

  /// Clone self with a new state.
  #[inline]
  pub fn clone_with(&self, state: State) -> Self {
    Self {
      stateless: self.stateless.clone(),
      instant: self.instant.clone(),
      state,
    }
  }

  /// Consume self, return a new lexer with the same actions and a new text.
  /// [`Self::state`] and [`Self::state`] will be reset to default.
  #[inline]
  pub fn reload<'new_text>(self, text: &'new_text str) -> Lexer<'new_text, Kind, State>
  where
    State: Default,
  {
    self.reload_with(text, State::default())
  }

  /// Consume self, return a new lexer with the same actions, a new text and the given state.
  /// [`Self::state`] will be reset to default.
  #[inline]
  pub fn reload_with<'new_text>(
    self,
    text: &'new_text str,
    state: State,
  ) -> Lexer<'new_text, Kind, State> {
    Lexer::new(self.stateless, state, text)
  }

  /// Take a snapshot of the current [`Self::state`] and [`Self::instant`].
  #[inline]
  pub fn snapshot(&self) -> Snapshot<'text, State>
  where
    State: Clone,
  {
    Snapshot {
      state: self.state.clone(),
      instant: self.instant.clone(),
    }
  }

  /// Restore [`Self::state`] and [`Self::instant`] from a [`Snapshot`].
  #[inline]
  pub fn restore(&mut self, snapshot: Snapshot<'text, State>) {
    self.state = snapshot.state;
    self.instant = snapshot.instant;
  }

  /// Clone self with the provided [`Snapshot`].
  #[inline]
  pub fn clone_with_snapshot(&self, snapshot: Snapshot<'text, State>) -> Self {
    Self {
      stateless: self.stateless.clone(),
      state: snapshot.state,
      instant: snapshot.instant,
    }
  }

  /// Peek the next token with the default options, without updating
  /// [`Self::state`] and [`Self::state`].
  ///
  /// [`Self::state`] will be cloned and returned.
  #[inline]
  pub fn peek(&self) -> (LexOutput<Token<Kind>, ()>, State)
  where
    State: Clone,
  {
    self.peek_with_options(LexOptions::new())
  }

  /// Peek the next token with custom options, without updating
  /// [`Self::state`] and [`Self::state`].
  ///
  /// [`Self::state`] will be cloned and returned.
  #[inline]
  pub fn peek_with<'expect_literal, Fork: LexOptionsFork>(
    &self,
    options_builder: impl FnOnce(
      LexOptions<'expect_literal, Kind, ()>,
    ) -> LexOptions<'expect_literal, Kind, Fork>,
  ) -> (
    LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType>,
    State,
  )
  where
    State: Clone,
  {
    self.peek_with_options(options_builder(LexOptions::new()))
  }

  /// Peek the next token with custom options, without updating
  /// [`Self::state`] and [`Self::state`].
  ///
  /// [`Self::state`] will be cloned and returned.
  pub fn peek_with_options<'expect_literal, Fork: LexOptionsFork>(
    &self,
    options: impl Into<LexOptions<'expect_literal, Kind, Fork>>,
  ) -> (
    LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType>,
    State,
  )
  where
    State: Clone,
  {
    let (output, new_state) = self.stateless.peek_with_options(
      self.instant.text(),
      StatelessLexOptions {
        start: self.instant.digested(),
        state: &self.state,
        base: options.into(),
      },
    );

    // don't update lexer state

    (output, new_state)
  }

  /// Try to yield the next token with the default options.
  /// [`Self::state`] and [`Self::state`] will be updated.
  #[inline]
  pub fn lex(&mut self) -> LexOutput<Token<Kind>, ()> {
    self.lex_with_options(LexOptions::new())
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::state`] will be updated.
  #[inline]
  pub fn lex_with<'expect_literal, Fork: LexOptionsFork>(
    &mut self,
    options_builder: impl FnOnce(
      LexOptions<'expect_literal, Kind, ()>,
    ) -> LexOptions<'expect_literal, Kind, Fork>,
  ) -> LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType> {
    self.lex_with_options(options_builder(LexOptions::new()))
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::state`] will be updated.
  pub fn lex_with_options<'expect_literal, Fork: LexOptionsFork>(
    &mut self,
    options: impl Into<LexOptions<'expect_literal, Kind, Fork>>,
  ) -> LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType> {
    let output = Self::lex_with_stateless(
      &self.stateless,
      &self.instant,
      &mut self.state,
      options.into(),
    );

    // update state
    self.instant.digest(output.digested);

    output
  }

  /// Digest the next `n` chars and optionally set [`Self::state`].
  /// The caller should make sure `n` is smaller than the rest text length.
  #[inline]
  pub fn digest_with(&mut self, n: usize, state: impl Into<Option<State>>) -> &mut Self {
    self.instant.digest(n);
    if let Some(state) = state.into() {
      self.state = state;
    }
    self
  }

  /// Digest the next `n` chars and set [`Self::state`] to default.
  /// The caller should make sure `n` is smaller than the rest text length.
  #[inline]
  pub fn digest(&mut self, n: usize) -> &mut Self
  where
    State: Default,
  {
    self.digest_with(n, State::default())
  }

  /// Lex with muted actions and the provided options.
  /// Returns [`None`] if the lexer is already trimmed.
  pub fn trim_with_options(&mut self, options: TrimOptions) -> Option<TrimOutput> {
    // return None if already trimmed
    if self.instant.trimmed() {
      return None;
    }

    let res = self.stateless.trim_with_options(
      self.instant.text(),
      StatelessTrimOptions {
        start: self.instant.digested(),
        state: &mut self.state,
        base: options,
      },
    );

    self.instant.trim(res.digested);

    Some(res)
  }

  /// Lex with muted actions and the provided options.
  /// Returns [`None`] if the lexer is already trimmed.
  #[inline]
  pub fn trim_with(&mut self, f: impl FnOnce(TrimOptions) -> TrimOptions) -> Option<TrimOutput> {
    self.trim_with_options(f(TrimOptions))
  }

  /// Lex with muted actions and the default options.
  /// Returns [`None`] if the lexer is already trimmed.
  #[inline]
  pub fn trim(&mut self) -> Option<TrimOutput> {
    self.trim_with_options(TrimOptions)
  }

  #[inline]
  fn lex_with_stateless<'expect_literal, Fork: LexOptionsFork>(
    stateless: &Rc<StatelessLexer<Kind, State>>,
    instant: &Instant<'text>,
    state: &mut State,
    options: LexOptions<'expect_literal, Kind, Fork>,
  ) -> LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType> {
    stateless.lex_with_options(
      instant.text(),
      StatelessLexOptions {
        start: instant.digested(),
        state,
        base: options,
      },
    )
  }
}

/// A helper trait to convert common types into a lexer.
pub trait IntoLexer<Kind, State>: Sized {
  /// Consume self, build a [`Lexer`] with the provided `state` and `text`.
  fn into_lexer_with(self, state: State, text: &str) -> Lexer<Kind, State>;

  /// Consume self, build a [`Lexer`] with the provided `text` and the default `State`.
  #[inline]
  fn into_lexer(self, text: &str) -> Lexer<Kind, State>
  where
    State: Default,
  {
    self.into_lexer_with(State::default(), text)
  }
}

impl<Kind, State> IntoLexer<Kind, State> for Rc<StatelessLexer<Kind, State>> {
  #[inline]
  fn into_lexer_with(self, state: State, text: &str) -> Lexer<Kind, State> {
    Lexer::new(self, state, text)
  }
}

impl<Kind, State> IntoLexer<Kind, State> for StatelessLexer<Kind, State> {
  #[inline]
  fn into_lexer_with(self, state: State, text: &str) -> Lexer<Kind, State> {
    Rc::new(self).into_lexer_with(state, text)
  }
}
