//! Lexical analysis.
//!
//! ## Getting Started
//!
//! Here is the recommended order of learning this module:
//!
//! - [`token`]
//! - [`action`]
//! - [`instant`]
//! - [`snapshot`]
//! - [`expectation`]
//! - [`re_lex`]
//! - [`fork`]
//! - [`options`]
//! - [`output`]
//! - [`self`]
//! - [`builder`]
//! - [`into`]
//! - [`stateless`]
//! - [`position`]

pub mod action;
pub mod builder;
pub mod expectation;
pub mod fork;
pub mod instant;
pub mod into;
pub mod options;
pub mod output;
pub mod position;
pub mod re_lex;
pub mod snapshot;
pub mod stateless;
pub mod token;

pub use builder::*;
pub use expectation::*;
pub use fork::*;
pub use instant::*;
pub use into::*;
pub use options::*;
pub use output::*;
pub use position::*;
pub use re_lex::*;
pub use snapshot::*;
pub use stateless::*;
pub use token::*;

use std::rc::Rc;

/// This is the "stateful" lexer, it manages the [`Instant`],
/// the [`State`](crate::lexer::action::ActionInput::state)
/// and the [`Heap`](crate::lexer::action::ActionInput::heap).
///
/// If you want a stateless experience, you can use [`StatelessLexer`].
///
/// To create a lexer, see [`LexerBuilder`](crate::lexer::builder::LexerBuilder).
#[derive(Debug)]
pub struct Lexer<'a, 'text, Kind, State = (), Heap = ()> {
  /// See [`ActionInput::state`](crate::lexer::action::ActionInput::state).
  ///
  /// You can mutate this directly if needed.
  pub state: State,
  /// See [`ActionInput::heap`](crate::lexer::action::ActionInput::heap).
  ///
  /// You can mutate this directly if needed.
  pub heap: Heap,

  // use Rc so that this is clone-able
  stateless: Rc<StatelessLexer<'a, Kind, State, Heap>>,
  instant: Instant<'text>,
}

impl<'a, 'text, Kind, State: Clone, Heap: Clone> Clone for Lexer<'a, 'text, Kind, State, Heap> {
  /// Clone the lexer, including [`Self::state`] and [`Self::heap`].
  /// # Performance
  /// Cloning the [`Self::heap`] might be expensive, you should use [`Lexer::snapshot`] to avoid cloning [`Self::heap`],
  /// and re-use one `heap` as much as possible.
  /// If you want to prevent users from cloning this, don't implement [`Clone`] for `Heap`.
  #[inline]
  fn clone(&self) -> Self {
    Self {
      state: self.state.clone(),
      heap: self.heap.clone(),
      stateless: self.stateless.clone(),
      instant: self.instant.clone(),
    }
  }
}

impl<'a, 'text, Kind, State, Heap> Lexer<'a, 'text, Kind, State, Heap> {
  /// Create a new lexer with the given stateless lexer, state, heap and text.
  ///
  /// See [`LexerBuilder`](crate::lexer::builder::LexerBuilder) to create a lexer.
  /// You can also use [`IntoLexer`] to convert some types to a lexer.
  #[inline]
  pub const fn new(
    stateless: Rc<StatelessLexer<'a, Kind, State, Heap>>,
    state: State,
    heap: Heap,
    text: &'text str,
  ) -> Self {
    Self {
      state,
      heap,
      stateless,
      instant: Instant::new(text),
    }
  }

  /// Get the stateless lexer.
  #[inline]
  pub const fn stateless(&self) -> &Rc<StatelessLexer<'a, Kind, State, Heap>> {
    &self.stateless
  }
  /// See [`Instant`].
  /// You are not allowed to mutate this directly.
  #[inline]
  pub const fn instant(&self) -> &Instant<'text> {
    &self.instant
  }

  /// Consume self, return a new lexer with the same actions and a new text.
  ///
  /// [`Self::instant`] and [`Self::state`] will be reset to default.
  /// [`Self::heap`] won't change.
  #[inline]
  pub fn reload<'new_text>(self, text: &'new_text str) -> Lexer<'a, 'new_text, Kind, State, Heap>
  where
    State: Default,
  {
    self.reload_with(text, State::default())
  }

  /// Consume self, return a new lexer with the same actions, a new text and the given state.
  /// [`Self::heap`] won't change.
  #[inline]
  pub fn reload_with<'new_text>(
    self,
    text: &'new_text str,
    state: State,
  ) -> Lexer<'a, 'new_text, Kind, State, Heap> {
    Lexer::new(self.stateless, state, self.heap, text)
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

  /// Peek the next token with the default options, without updating
  /// [`Self::instant`] and [`Self::state`].
  ///
  /// [`Self::state`] will be cloned and returned.
  /// [`Self::heap`] might be mutated.
  #[inline]
  pub fn peek(&mut self) -> (LexOutput<Token<Kind>, ()>, State)
  where
    State: Clone,
  {
    self.peek_with_options(LexOptions::new())
  }

  /// Peek the next token with custom options, without updating
  /// [`Self::instant`] and [`Self::state`].
  ///
  /// [`Self::state`] will be cloned and returned.
  /// [`Self::heap`] might be mutated.
  #[inline]
  pub fn peek_with<'literal, Fork: LexOptionsFork>(
    &mut self,
    options_builder: impl FnOnce(LexOptions<'literal, Kind, ()>) -> LexOptions<'literal, Kind, Fork>,
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
  /// [`Self::instant`] and [`Self::state`].
  ///
  /// [`Self::state`] will be cloned and returned.
  /// [`Self::heap`] might be mutated.
  pub fn peek_with_options<'literal, Fork: LexOptionsFork>(
    &mut self,
    options: impl Into<LexOptions<'literal, Kind, Fork>>,
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
        heap: &mut self.heap,
        base: options.into(),
      },
    );

    // don't update state

    (output, new_state)
  }

  /// Try to yield the next token with the default options.
  /// [`Self::instant`] will be updated.
  #[inline]
  pub fn lex(&mut self) -> LexOutput<Token<Kind>, ()> {
    self.lex_with_options(LexOptions::new())
  }

  /// Try to yield the next token with custom options.
  /// [`Self::instant`] will be updated.
  #[inline]
  pub fn lex_with<'literal, Fork: LexOptionsFork>(
    &mut self,
    options_builder: impl FnOnce(LexOptions<'literal, Kind, ()>) -> LexOptions<'literal, Kind, Fork>,
  ) -> LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType> {
    self.lex_with_options(options_builder(LexOptions::new()))
  }

  /// Try to yield the next token with custom options.
  /// [`Self::instant`] will be updated.
  pub fn lex_with_options<'literal, Fork: LexOptionsFork>(
    &mut self,
    options: impl Into<LexOptions<'literal, Kind, Fork>>,
  ) -> LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType> {
    let options = options.into();

    let output = self.stateless.lex_with_options(
      self.instant.text(),
      StatelessLexOptions {
        start: self.instant.digested(),
        state: &mut self.state,
        heap: &mut self.heap,
        base: options,
      },
    );

    // update state
    self.instant.digest(output.digested);

    output
  }

  /// Lex with muted actions and the default options.
  /// Returns [`None`] if the lexer is already trimmed.
  /// [`Self::instant`] will be updated.
  #[inline]
  pub fn trim(&mut self) -> Option<TrimOutput> {
    self.trim_with_options(TrimOptions)
  }

  /// Lex with muted actions and the provided options.
  /// Returns [`None`] if the lexer is already trimmed.
  /// [`Self::instant`] will be updated.
  #[inline]
  pub fn trim_with(&mut self, f: impl FnOnce(TrimOptions) -> TrimOptions) -> Option<TrimOutput> {
    self.trim_with_options(f(TrimOptions))
  }

  /// Lex with muted actions and the provided options.
  /// Returns [`None`] if the lexer is already trimmed.
  /// [`Self::instant`] will be updated.
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
        heap: &mut self.heap,
        base: options,
      },
    );

    // update state
    self.instant.trim(res.digested);

    Some(res)
  }

  /// Digest the next `n` chars and set [`Self::state`] to the default.
  ///
  /// The caller should make sure `n` is smaller than the rest text length,
  /// this will be checked using [`debug_assert`].
  #[inline]
  pub fn digest(&mut self, n: usize) -> &mut Self
  where
    State: Default,
  {
    self.digest_with(n, State::default())
  }

  /// Digest the next `n` chars and optionally set [`Self::state`].
  ///
  /// The caller should make sure `n` is smaller than the rest text length,
  /// this will be checked using [`debug_assert`].
  #[inline]
  pub fn digest_with(&mut self, n: usize, state: impl Into<Option<State>>) -> &mut Self {
    self.instant.digest(n);
    if let Some(state) = state.into() {
      self.state = state;
    }
    self
  }
}
