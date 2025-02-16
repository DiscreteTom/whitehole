//! Manage [`Context::state`], [`Context::heap`] and the parsing progress.
//!
//! # Build the Parser
//!
//! See [`Builder`].
//!
//! # State and Heap
//!
//! Parser will manage [`Parser::state`] which is accessible by actions
//! so that you can make the parser stateful.
//!
//! For example, if a language supports regex literal,
//! then `/` can be a division operator or the start of a regex literal.
//! You can use stateful parsing to switch between these two modes.
//!
//! ```
//! use whitehole::{combinator::{eat, next}, parser::Parser};
//!
//! #[derive(PartialEq, Eq, Debug)]
//! enum Mode {
//!   Normal,
//!   Regex,
//! }
//!
//! let whitespaces = (eat(' ') | eat('\n')) * (1..);
//! let identifier = eat("a");
//! let number = next(|c| c.is_ascii_digit());
//!
//! // in normal mode, '/' is a division operator
//! let div = eat('/').when(|_, ctx| *ctx.state == Mode::Normal);
//!
//! // after '=', switch to regex mode
//! let assign = eat('=').then(|_, ctx| *ctx.state = Mode::Regex);
//!
//! // in regex mode, '/' is the start of a regex literal
//! let regex = eat("/123/")
//!   .when(|_, ctx| *ctx.state == Mode::Regex)
//!   // after the regex literal, switch back to normal mode
//!   .then(|_, ctx| *ctx.state = Mode::Normal);
//!
//! let entry = whitespaces | identifier | number | assign | div | regex;
//!
//! let mut parser = Parser::builder()
//!   .state(Mode::Normal)
//!   .entry(entry)
//!   .build("a=/123/ \n a=1/2");
//!
//! assert_eq!(parser.state, Mode::Normal);
//! assert_eq!(parser.next().unwrap().digested, 1); // "a"
//! assert_eq!(parser.next().unwrap().digested, 1); // "="
//! assert_eq!(parser.state, Mode::Regex);
//! assert_eq!(parser.next().unwrap().digested, 5); // "/123/"
//! assert_eq!(parser.state, Mode::Normal);
//! assert_eq!(parser.next().unwrap().digested, 3); // " \n "
//! assert_eq!(parser.next().unwrap().digested, 1); // "a"
//! assert_eq!(parser.next().unwrap().digested, 1); // "="
//! assert_eq!(parser.state, Mode::Regex);
//! assert_eq!(parser.next().unwrap().digested, 1); // "1"
//! parser.state = Mode::Normal; // manually switch back to normal mode
//! assert_eq!(parser.next().unwrap().digested, 1); // "/"
//! assert_eq!(parser.next().unwrap().digested, 1); // "2"
//! assert!(parser.next().is_none());
//! ```
//!
//! For non-state data, you can use [`Parser::heap`] which is also accessible by actions.
//! You can use the heap to pass data to actions or store data that is generated by actions.
//!
//! See [`Context::state`] and [`Context::heap`] for more information.
//!
//! # Parse and Peek
//!
//! You can use [`Parser::next`] to try to yield the next [`Output`].
//!
//! If you just want to peek the next output without updating the parser,
//! you can use [`Parser::peek`] instead.
//!
//! ```
//! use whitehole::{combinator::eat, parser::Parser};
//!
//! let mut parser = Parser::builder()
//!   .entry(eat("123"))
//!   .build("123");
//!
//! // peek will clone the state
//! let (output, state) = parser.peek();
//! ```
//!
//! # Iter
//!
//! [`Parser`] implements [`Iterator`] so you can use it in a for loop
//! or with any iterator methods.
//!
//! ```
//! use whitehole::{combinator::eat, parser::Parser};
//!
//! let factory = || {
//!   Parser::builder()
//!     .entry(eat("123"))
//!     .build("123123123")
//! };
//!
//! // for loop
//! let mut parser = factory();
//! for o in &mut parser {
//!   assert_eq!(o.digested, 3);
//! }
//!
//! // iterator methods
//! let mut parser = factory();
//! for (_, o) in (&mut parser).enumerate() {
//!   assert_eq!(o.digested, 3);
//! }
//! ```
//!
//! # Instant
//!
//! [`Parser`] uses [`Instant`] to manage the parsing progress.
//! You can use [`Parser::instant`] to access the current progress.
//!
//! ```
//! use whitehole::{combinator::eat, parser::Parser};
//!
//! let mut parser = Parser::builder()
//!  .entry(eat("123"))
//!  .build("123123");
//!
//! assert_eq!(parser.instant().text(), "123123");
//! assert_eq!(parser.instant().rest(), "123123");
//! assert_eq!(parser.instant().digested(), 0);
//!
//! parser.next();
//! assert_eq!(parser.instant().rest(), "123");
//! assert_eq!(parser.instant().digested(), 3);
//!
//! parser.next();
//! assert_eq!(parser.instant().rest(), "");
//! assert_eq!(parser.instant().digested(), 6);
//! ```
//!
//! You can't modify [`Parser::instant`] directly.
//! If you want to parse an other text, use [`Parser::reload`] or [`Parser::reload_with`] instead,
//! these methods will reset the instant to default.
//!
//! # Snapshots
//!
//! [`Parser`] is clone-able when your entry action, state and heap are clone-able.
//!
//! However, if you have a heap, cloning the parser might be expensive.
//! In this case, you can use [`Parser::snapshot`] and [`Parser::restore`] to save and restore the state and instant.
//!
//! ```
//! use whitehole::{combinator::eat, parser::{Parser, Snapshot}};
//!
//! let mut parser = Parser::builder()
//!   .entry(eat("123"))
//!   .build("123");
//!
//! let snapshot = parser.snapshot();
//! parser.next();
//! assert_eq!(parser.instant().digested(), 3);
//!
//! parser.restore(snapshot);
//! assert_eq!(parser.instant().digested(), 0);
//! ```
//!
//! It's like [`Parser::peek`], but you can save as many snapshots as you want.
//!
//! # External Digestion
//!
//! You can use [`Parser::digest_unchecked`] and [`Parser::digest_with_unchecked`] to digest the next `n` bytes.
//! These methods are useful when you want to digest some bytes from outside of the parser,
//! e.g. in error handling or recovery.
//!
//! ```
//! use whitehole::{combinator::eat, parser::Parser};
//!
//! let mut parser = Parser::builder()
//!   .entry(eat("123"))
//!   .build("a123");
//!
//! assert!(parser.next().is_none());
//!
//! // enter "panic mode", digest the next char from outside
//! unsafe { parser.digest_unchecked(parser.instant().rest().chars().next().unwrap().len_utf8()) };
//! assert_eq!(parser.instant().rest(), "123");
//!
//! // now we can try to yield again
//! assert!(parser.next().is_some());
//! ```

mod builder;
mod snapshot;

pub use builder::*;
pub use snapshot::*;

use crate::{
  action::{Action, Context, Output},
  digest::Digest,
  instant::Instant,
};
use std::{ops::RangeFrom, slice::SliceIndex};

/// Manage [`Context::state`], [`Context::heap`] and the parsing progress.
///
/// See the [module-level documentation](self) for more.
#[derive(Debug, Clone)]
pub struct Parser<T, TextRef, State = (), Heap = ()> {
  /// See [`Context::state`].
  /// You can mutate this directly if needed.
  pub state: State,
  /// See [`Context::heap`].
  /// You can mutate this directly if needed.
  pub heap: Heap,

  /// See [`Self::instant`].
  instant: Instant<TextRef>,
  /// See [`Self::entry`].
  entry: T,
}

impl Parser<(), &str> {
  /// Create a parser builder with default settings.
  #[inline]
  pub const fn builder() -> Builder<()> {
    Builder::new()
  }
}

impl<T, TextRef, State, Heap> Parser<T, TextRef, State, Heap> {
  /// The entry action.
  #[inline]
  pub const fn entry(&self) -> &T {
    &self.entry
  }

  /// See [`Instant`].
  /// You are not allowed to mutate this directly.
  #[inline]
  pub const fn instant(&self) -> &Instant<TextRef> {
    &self.instant
  }

  /// Consume self, return a new instance with the same action and a new text.
  ///
  /// [`Self::instant`] and [`Self::state`] will be reset to default.
  /// [`Self::heap`] won't change.
  #[inline]
  pub fn reload<NewText: ?Sized>(self, text: &NewText) -> Parser<T, &NewText, State, Heap>
  where
    State: Default,
  {
    self.reload_with(State::default(), text)
  }

  /// Consume self, return a new instance with the same action, a new text and an optional new state.
  ///
  /// If the state is not provided, current [`Self::state`] will be kept.
  /// [`Self::instant`] will be reset to default.
  /// [`Self::heap`] won't change.
  #[inline]
  pub fn reload_with<NewText: ?Sized>(
    self,
    state: impl Into<Option<State>>,
    text: &NewText,
  ) -> Parser<T, &NewText, State, Heap> {
    Parser {
      entry: self.entry,
      heap: self.heap,
      state: state.into().unwrap_or(self.state),
      instant: Instant::new(text),
    }
  }

  /// Take a snapshot of the current [`Self::state`] and [`Self::instant`].
  #[inline]
  pub fn snapshot(&self) -> Snapshot<TextRef, State>
  where
    TextRef: Clone,
    State: Clone,
  {
    Snapshot {
      state: self.state.clone(),
      instant: self.instant.clone(),
    }
  }

  /// Restore [`Self::state`] and [`Self::instant`] from a [`Snapshot`].
  #[inline]
  pub fn restore(&mut self, snapshot: Snapshot<TextRef, State>) {
    self.state = snapshot.state;
    self.instant = snapshot.instant;
  }
}

impl<T, Text: ?Sized, State, Heap> Parser<T, &Text, State, Heap> {
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
    Text: Digest,
    State: Default,
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    self.digest_with_unchecked(State::default(), n)
  }

  /// Digest the next `n` bytes and optionally set [`Self::state`].
  /// # Safety
  /// See [`Instant::digest_unchecked`].
  #[inline]
  pub unsafe fn digest_with_unchecked(&mut self, state: impl Into<Option<State>>, n: usize)
  where
    Text: Digest,
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    self.instant.digest_unchecked(n);
    if let Some(state) = state.into() {
      self.state = state;
    }
  }

  /// Try to yield the next [`Output`] without updating [`Self::instant`] and [`Self::state`].
  /// [`Self::state`] will be cloned and returned.
  /// Return [`None`] if the action rejects.
  #[inline]
  pub fn peek(&mut self) -> (Option<Output<T::Value>>, State)
  where
    T: Action<Text, State, Heap>,
    State: Clone,
  {
    let mut tmp_state = self.state.clone();
    (
      self.entry.exec(
        &self.instant,
        Context {
          state: &mut tmp_state,
          heap: &mut self.heap,
        },
      ),
      tmp_state,
    )
  }
}

impl<T: Action<Text, State, Heap>, Text: ?Sized + Digest, State, Heap> Iterator
  for Parser<T, &Text, State, Heap>
where
  RangeFrom<usize>: SliceIndex<Text, Output = Text>,
{
  type Item = Output<T::Value>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self
      .entry
      .exec(
        &self.instant,
        Context {
          state: &mut self.state,
          heap: &mut self.heap,
        },
      )
      .inspect(|output| unsafe { self.instant.digest_unchecked(output.digested) })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::eat;
  use std::rc::Rc;

  #[test]
  fn parser_builder() {
    let parser = Parser::builder()
      .state(123)
      .heap(123)
      .entry(eat("123"))
      .build("123");
    assert_eq!(parser.state, 123);
    assert_eq!(parser.heap, 123);
    assert_eq!(parser.instant().text(), "123");
  }

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
        .exec(
          &Instant::new("123"),
          Context {
            state: &mut 0,
            heap: &mut 0
          }
        )
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
    parser.next();
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
    parser.next();
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
    parser.next();
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
    let output = parser.next().unwrap();
    assert_eq!(output.digested, 3);
    assert_eq!(output.value, ());
    assert_eq!(parser.instant().digested(), 3);
    assert_eq!(parser.instant().rest(), "");
    assert!(parser.next().is_none());
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
    assert!(parser.next().is_some());
  }

  #[test]
  fn str_in_heap() {
    let text = "123".to_string();
    let mut parser = Parser {
      state: 123,
      heap: &text,
      instant: Instant::new(text.as_str()),
      entry: eat(text.as_str()),
    };
    assert!(parser.next().is_some());
  }

  #[test]
  fn parser_iterator_in_for_loop() {
    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123123123"),
      entry: eat("123"),
    };
    for o in &mut parser {
      assert_eq!(o.digested, 3);
    }
    assert_eq!(parser.instant().digested(), 9);
  }

  #[test]
  fn parser_iterator_with_iter_methods() {
    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123123123"),
      entry: eat("123"),
    };
    for (_, o) in (&mut parser).enumerate() {
      assert_eq!(o.digested, 3);
    }
    assert_eq!(parser.instant().digested(), 9);
  }
}
