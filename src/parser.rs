//! Use [`Builder`] to build a parser.

mod builder;
mod snapshot;

pub use builder::*;
pub use snapshot::*;

use crate::{
  action::{Action, Input, Output},
  instant::Instant,
};

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
  /// The entry action.
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

  /// Consume self, return a new instance with the same action and a new text.
  /// [`Self::instant`] and [`Self::state`] will be reset to default.
  /// [`Self::heap`] won't change.
  #[inline]
  pub fn reload(self, text: &str) -> Parser<T>
  where
    T::State: Default,
  {
    self.reload_with(T::State::default(), text)
  }

  /// Consume self, return a new instance with the same action, a new text and an optional new state.
  /// If the state is not provided, current [`Self::state`] will be kept.
  /// [`Self::instant`] will be reset to default.
  /// [`Self::heap`] won't change.
  #[inline]
  pub fn reload_with(self, state: impl Into<Option<T::State>>, text: &str) -> Parser<T> {
    Parser {
      entry: self.entry,
      heap: self.heap,
      state: state.into().unwrap_or(self.state),
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

  /// Digest the next `n` bytes and set [`Self::state`] to the default.
  ///
  /// Usually when you digest some bytes from outside of the parser
  /// (e.g. by an error recovery strategy),
  /// the state should be reset to the default.
  /// If you want to keep the state, use [`Self::digest_with_unchecked`] instead.
  /// # Safety
  /// See [`Instant::digest_unchecked`].
  #[inline]
  pub unsafe fn digest_unchecked(&mut self, n: usize)
  where
    T::State: Default,
  {
    self.digest_with_unchecked(T::State::default(), n)
  }

  /// Digest the next `n` chars and optionally set [`Self::state`].
  /// # Safety
  /// See [`Instant::digest_unchecked`].
  #[inline]
  pub unsafe fn digest_with_unchecked(&mut self, state: impl Into<Option<T::State>>, n: usize) {
    self.instant.digest_unchecked(n);
    if let Some(state) = state.into() {
      self.state = state;
    }
  }

  /// Try to yield the next [`Output`].
  /// Return [`None`] if the text is already fully digested
  /// or the action rejects.
  #[inline]
  pub fn parse(&mut self) -> Option<Output<T::Value>> {
    self
      .entry
      .exec(Input::new(
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
  /// or the action rejects.
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
      .and_then(|input| self.entry.exec(input)),
      tmp_state,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::eat;
  use std::rc::Rc;

  #[test]
  fn parser_clone() {
    let parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: Rc::new(eat("123")),
    }
    .clone();
    assert_eq!(parser.state, 123);
    assert_eq!(parser.heap, 123);
  }

  #[test]
  fn parser_getters() {
    let parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    assert_eq!(
      parser
        .entry()
        .exec(Input::new("123", 0, &mut 0, &mut 0).unwrap())
        .unwrap()
        .digested,
      3
    );
    assert_eq!(parser.instant().digested(), 0);
  }

  #[test]
  fn parser_reload() {
    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    parser.parse();
    assert_eq!(parser.instant().digested(), 3);
    assert_eq!(parser.instant().rest(), "");
    let parser = parser.reload("456");
    assert_eq!(parser.instant().text(), "456");
    assert_eq!(parser.instant().rest(), "456");
    assert_eq!(parser.instant().digested(), 0);
    assert_eq!(parser.state, 0);
    assert_eq!(parser.heap, 123);
  }

  #[test]
  fn parser_reload_with() {
    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    parser.parse();
    assert_eq!(parser.instant().digested(), 3);
    assert_eq!(parser.instant().rest(), "");
    let parser = parser.reload_with(None, "456");
    assert_eq!(parser.instant().text(), "456");
    assert_eq!(parser.instant().rest(), "456");
    assert_eq!(parser.instant().digested(), 0);
    assert_eq!(parser.state, 123);
    assert_eq!(parser.heap, 123);
  }

  #[test]
  fn parser_snapshot_restore() {
    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    parser.parse();
    let snapshot = parser.snapshot();
    assert_eq!(snapshot.state, 123);
    assert_eq!(snapshot.instant().text(), "123");
    assert_eq!(snapshot.instant().digested(), 3);
    assert_eq!(snapshot.instant().rest(), "");

    let mut parser = Parser {
      state: 0,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    parser.restore(snapshot);
    assert_eq!(parser.state, 123);
    assert_eq!(parser.instant().text(), "123");
    assert_eq!(parser.instant().digested(), 3);
    assert_eq!(parser.instant().rest(), "");
  }

  #[test]
  fn parser_digest_unchecked() {
    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    unsafe { parser.digest_unchecked(1) };
    assert_eq!(parser.state, 0);
    assert_eq!(parser.instant().digested(), 1);
    assert_eq!(parser.instant().rest(), "23");
  }

  #[test]
  #[should_panic]
  fn parser_digest_unchecked_overflow() {
    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    unsafe { parser.digest_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn parser_digest_unchecked_invalid_code_point() {
    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("好"),
      entry: eat("123"),
    };
    unsafe { parser.digest_unchecked(1) };
  }

  #[test]
  fn parser_digest_with_unchecked() {
    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    unsafe { parser.digest_with_unchecked(None, 1) };
    assert_eq!(parser.state, 123);
    assert_eq!(parser.instant().digested(), 1);
    assert_eq!(parser.instant().rest(), "23");

    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    unsafe { parser.digest_with_unchecked(456, 1) };
    assert_eq!(parser.state, 456);
    assert_eq!(parser.instant().digested(), 1);
    assert_eq!(parser.instant().rest(), "23");
  }

  #[test]
  fn parser_parse() {
    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    let output = parser.parse().unwrap();
    assert_eq!(output.digested, 3);
    assert_eq!(output.value, ());
    assert_eq!(parser.instant().digested(), 3);
    assert_eq!(parser.instant().rest(), "");
    assert!(parser.parse().is_none());
  }

  #[test]
  fn parser_peek() {
    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    let (output, state) = parser.peek();
    let output = output.unwrap();
    assert_eq!(state, 123);
    assert_eq!(output.digested, 3);
    assert_eq!(output.value, ());
    assert_eq!(parser.instant().digested(), 0);
    assert_eq!(parser.instant().rest(), "123");
    assert!(parser.parse().is_some());
  }
}
