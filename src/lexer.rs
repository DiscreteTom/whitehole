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

use crate::lexer::instant::Instant;
use fork::{ForkOutputFactory, LexOptionsFork};
use options::{LexOptions, TrimOptions};
use output::{LexOutput, TrimOutput};
use snapshot::Snapshot;
use stateless::{StatelessLexOptions, StatelessLexer, StatelessTrimOptions};
use std::rc::Rc;
use token::Token;

/// This is the "stateful" lexer, it manages the [`Instant`] and the `State`.
///
/// The [`Instant`] is responsible to manage the text and the progress of the lexer.
/// The `State` is provided by you and can be accessed by [`Action`](crate::lexer::action::Action)s
/// to realize stateful lexing.
///
/// If you want a stateless experience, you can use [`StatelessLexer`].
///
/// To create a lexer, you should use [`LexerBuilder`](crate::lexer::builder::LexerBuilder).
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
pub struct Lexer<'a, 'text, Kind, State = (), Heap = ()> {
  /// You can mutate this directly if needed.
  pub state: State,
  /// You can mutate this directly if needed.
  pub heap: Heap,

  // use Rc so that this is clone-able
  stateless: Rc<StatelessLexer<'a, Kind, State, Heap>>,
  instant: Instant<'text>,
}

impl<'a, 'text, Kind, State: Clone, Heap: Clone> Clone for Lexer<'a, 'text, Kind, State, Heap> {
  /// Clone the lexer, including [`Self::state`] and [`Self::heap`].
  /// # Performance
  /// Cloning the `Heap` might be expensive, you should use [`Lexer::snapshot`] to just clone the state,
  /// and re-use one `heap` as much as possible.
  /// If you want to prevent users from cloning [`Self::heap`], don't implement [`Clone`] for `Heap`.
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
  /// Create a new lexer with the given stateless lexer, state and text.
  /// For most cases you should use [`LexerBuilder`](crate::lexer::builder::LexerBuilder)
  /// to create a lexer.
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
  /// Get the lexer state.
  /// You are not allowed to mutate the lexer state directly.
  #[inline]
  pub const fn instant(&self) -> &Instant<'text> {
    &self.instant
  }

  /// Consume self, return a new lexer with the same actions and a new text.
  /// [`Self::instant`] and [`Self::state`] will be reset to default.
  #[inline]
  pub fn reload<'new_text>(self, text: &'new_text str) -> Lexer<'a, 'new_text, Kind, State, Heap>
  where
    State: Default,
  {
    self.reload_with(text, State::default())
  }

  /// Consume self, return a new lexer with the same actions, a new text and the given state.
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
  /// [`Self::state`] and [`Self::state`].
  ///
  /// [`Self::state`] will be cloned and returned.
  #[inline]
  pub fn peek(&mut self) -> (LexOutput<Token<Kind>, ()>, State)
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
  /// [`Self::state`] and [`Self::state`].
  ///
  /// [`Self::state`] will be cloned and returned.
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
  pub fn lex_with<'literal, Fork: LexOptionsFork>(
    &mut self,
    options_builder: impl FnOnce(LexOptions<'literal, Kind, ()>) -> LexOptions<'literal, Kind, Fork>,
  ) -> LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType> {
    self.lex_with_options(options_builder(LexOptions::new()))
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::state`] will be updated.
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
        heap: &mut self.heap,
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
}
