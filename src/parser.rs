//! Use [`Builder`] to build a parser.

mod builder;
mod instant;
mod snapshot;

pub use builder::*;
pub use instant::*;
pub use snapshot::*;

use crate::action::{Action, Input, Output};

/// Manage [`Input::state`], [`Input::heap`] and the parsing progress.
#[derive(Debug)]
pub struct Parser<'text, T: Action> {
  /// See [`Input::state`](crate::action::Input::state).
  /// You can mutate this directly if needed.
  pub state: T::State,
  /// See [`Input::heap`](crate::action::Input::heap).
  /// You can mutate this directly if needed.
  pub heap: T::Heap,

  /// See [`Self::instant`].
  instant: Instant<'text>,
  /// See [`Self::entry`].
  entry: T,
}

impl<T: Action<State: Clone, Heap: Clone> + Clone> Clone for Parser<'_, T> {
  /// Clone the parser, including [`Self::state`] and [`Self::heap`].
  /// # Performance
  /// Cloning the [`Self::heap`] might be expensive, you should use [`Parser::snapshot`] to avoid cloning [`Self::heap`],
  /// and re-use one `heap` as much as possible.
  /// If you want to prevent users from cloning this, don't implement [`Clone`] for `Heap`.
  #[inline]
  fn clone(&self) -> Self {
    Self {
      state: self.state.clone(),
      heap: self.heap.clone(),
      entry: self.entry.clone(),
      instant: self.instant.clone(),
    }
  }
}

impl<'text, T: Action> Parser<'text, T> {
  /// The entry combinator.
  #[inline]
  pub const fn entry(&self) -> &T {
    &self.entry
  }

  /// See [`Instant`].
  /// You are not allowed to mutate this directly.
  #[inline]
  pub const fn instant(&self) -> &Instant<'text> {
    &self.instant
  }

  /// Consume self, return a new instance with the same combinator and a new text.
  /// [`Self::instant`] and [`Self::state`] will be reset to default.
  /// [`Self::heap`] won't change.
  #[inline]
  pub fn reload(self, text: &str) -> Parser<T>
  where
    T::State: Default,
  {
    self.reload_with(T::State::default(), text)
  }

  /// Consume self, return a new instance with the same combinator, a new text and the given state.
  /// [`Self::instant`] will be reset to default.
  /// [`Self::heap`] won't change.
  #[inline]
  pub fn reload_with(self, state: T::State, text: &str) -> Parser<T> {
    Parser {
      entry: self.entry,
      heap: self.heap,
      state,
      instant: Instant::new(text),
    }
  }

  /// Take a snapshot of the current [`Self::state`] and [`Self::instant`].
  #[inline]
  pub fn snapshot(&self) -> Snapshot<'text, T::State>
  where
    T::State: Clone,
  {
    Snapshot {
      state: self.state.clone(),
      instant: self.instant.clone(),
    }
  }

  /// Restore [`Self::state`] and [`Self::instant`] from a [`Snapshot`].
  #[inline]
  pub fn restore(&mut self, snapshot: Snapshot<'text, T::State>) {
    self.state = snapshot.state;
    self.instant = snapshot.instant;
  }

  // TODO
  // /// Digest the next `n` bytes and set [`Self::state`] to the default.
  // ///
  // /// Usually when you digest some bytes from outside of the parser
  // /// (e.g. by an error recovery strategy),
  // /// the state should be reset to the default.
  // /// If you want to keep the state, use [`Self::digest_with`] instead.
  // /// # Caveats
  // /// The caller should make sure `n` is no greater than the rest text length,
  // /// this will be checked using [`debug_assert`].
  // #[inline]
  // pub fn digest(&mut self, n: usize) -> &mut Self
  // where
  //   State: Default,
  // {
  //   self.digest_with(State::default(), n)
  // }

  // /// Digest the next `n` chars and optionally set [`Self::state`].
  // /// # Caveats
  // /// The caller should make sure `n` is no greater than the rest text length,
  // /// this will be checked using [`debug_assert`].
  // #[inline]
  // pub fn digest_with(&mut self, state: impl Into<Option<State>>, n: usize) -> &mut Self {
  //   debug_assert!(self.digested + n <= self.text.len());
  //   self.digested += n;
  //   if let Some(state) = state.into() {
  //     self.state = state;
  //   }
  //   self
  // }

  /// Try to yield the next [`Output`].
  /// Return [`None`] if the text is already fully digested
  /// or the combinator rejects.
  #[inline]
  pub fn parse(&mut self) -> Option<Output<T::Value>> {
    self
      .entry
      .exec(&mut Input::new(
        self.instant.rest(),
        self.instant.digested(),
        &mut self.state,
        &mut self.heap,
      )?)
      .inspect(|output| unsafe { self.instant.digest_unchecked(output.digested) })
  }

  /// Try to yield the next [`Output`] without updating [`Self::instant`] and [`Self::state`].
  /// [`Self::state`] will be cloned and returned.
  /// Return [`None`] if the text is already fully digested
  /// or the combinator rejects.
  #[inline]
  pub fn peek(&mut self) -> (Option<Output<T::Value>>, T::State)
  where
    T::State: Clone,
  {
    let mut tmp_state = self.state.clone();
    (
      Input::new(
        self.instant.text(),
        self.instant.digested(),
        &mut tmp_state,
        &mut self.heap,
      )
      .and_then(|mut input| self.entry.exec(&mut input)),
      tmp_state,
    )
  }
}

// TODO: tests
