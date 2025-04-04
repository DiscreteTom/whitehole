//! Manage the [`State`](Parser::state), [`Heap`](Parser::heap)
//! and the [parsing progress](Parser::instant).
//!
//! # Build the Parser
//!
//! See [`Builder`].
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
//! [`Parser`] implements [`Iterator`] so you can use it in a for-loop
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
//! // for-loop
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
//! # Progress
//!
//! ## Instant
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
//! assert_eq!(parser.instant.text(), "123123");
//! assert_eq!(parser.instant.rest(), "123123");
//! assert_eq!(parser.instant.digested(), 0);
//!
//! parser.next();
//! assert_eq!(parser.instant.rest(), "123");
//! assert_eq!(parser.instant.digested(), 3);
//!
//! parser.next();
//! assert_eq!(parser.instant.rest(), "");
//! assert_eq!(parser.instant.digested(), 6);
//! ```
//!
//! [`Parser::instant`] is public, you can mutate it directly if needed.
//! Be ware to keep the `State` and `Heap` in sync with the progress.
//!
//! If you want to parse an other text, use [`Parser::reload`] or [`Parser::reload_with`] instead,
//! these methods will reset the instant to default and restore `State` if needed.
//!
//! ## External Digestion
//!
//! You can update [`Parser::instant`] to digest from outside of the parser.
//! e.g. in error handling or recovery.
//!
//! Be ware to keep the `State` and `Heap` in sync with the progress.
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
//! let next_len = parser.instant.rest().chars().next().unwrap().len_utf8();
//! unsafe { parser.instant.digest_unchecked(next_len) };
//! assert_eq!(parser.instant.rest(), "123");
//!
//! // now we can try to yield again
//! assert!(parser.next().is_some());
//! ```
//!
//! ## Snapshots
//!
//! [`Parser`] is clone-able when your entry action, `State` and `Heap` are all clone-able.
//!
//! However, if you do have a `Heap` with heap allocation, cloning the parser might be expensive.
//! In this case, you can use [`Parser::snapshot`] and [`Parser::restore`]
//! to save and restore [`Parser::instant`] and [`Parser::state`].
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
//! assert_eq!(parser.instant.digested(), 3);
//!
//! parser.restore(snapshot);
//! assert_eq!(parser.instant.digested(), 0);
//! ```
//!
//! It's like [`Parser::peek`], but you can save as many snapshots as you want.
//!
//! # State and Heap
//!
//! Parser will manage [`Parser::state`] which is accessible by actions
//! so that you can make the parser stateful.
//!
//! For example, if a language supports regex literal,
//! then `/` can be a division operator or the start of a regex literal.
//! You can use stateful parsing to realize expectations like this:
//!
//! ```
//! use whitehole::{combinator::contextual, parser::Parser};
//!
//! #[derive(PartialEq, Eq, Debug)]
//! pub enum Expected {
//!   None,
//!   Expression,
//! }
//!
//! contextual!(Expected, ());
//!
//! # fn main() {
//! let whitespaces = || (eat(' ') | '\n') * (1..);
//! let identifier = || eat("a"); // just a dummy identifier
//! let number = || next(|c| c.is_ascii_digit());
//!
//! // by default, '/' is a division operator
//! let div = || eat('/').when(|input| *input.state == Expected::None);
//!
//! // after '=', expect an expression
//! let assign = || eat('=').then(|input| *input.state = Expected::Expression);
//!
//! // when an expression is expected, '/' is the start of a regex literal
//! let regex = || eat("/123/").when(|input| *input.state == Expected::Expression);
//!
//! // for simplicity, expression can only be a regex literal or a division between two numbers
//! let expression = || { regex() | (number() + div() + number()) }
//!   // after the expression, switch back to normal mode
//!   .then(|input| *input.state = Expected::None);
//!
//! let statement = || identifier() + assign() + expression();
//!
//! let entry = whitespaces() | statement();
//!
//! let mut parser = Parser::builder()
//!   .state(Expected::None)
//!   .entry(entry)
//!   .build("a=/123/ \n a=1/2");
//!
//! assert_eq!(parser.next().unwrap().digested, 7); // "a=/123/"
//! assert_eq!(parser.next().unwrap().digested, 3); // " \n "
//! assert_eq!(parser.next().unwrap().digested, 5); // "a=1/2"
//! assert!(parser.next().is_none());
//! # }
//! ```
//!
//! For non-state data, you can use [`Parser::heap`] which is also accessible by actions.
//! You can use the heap to pass data to actions or store data that is generated by actions.
//!
//! See [`Parser::state`] and [`Parser::heap`] for more information.

mod builder;
mod snapshot;

pub use builder::*;
pub use snapshot::*;

use crate::{
  action::{Action, Input, Output},
  combinator::Take,
  digest::Digest,
  instant::Instant,
};
use std::{ops::RangeFrom, slice::SliceIndex};

/// Manage the [`State`](Parser::state), [`Heap`](Parser::heap)
/// and the [parsing progress](Parser::instant).
///
/// See the [module-level documentation](self) for more.
#[derive(Debug)]
pub struct Parser<'text, T: Action> {
  /// The state of a stateful parser.
  ///
  /// With this, you can construct stateful parsers,
  /// while [`Action`]s remain stateless.
  ///
  /// All vars that control the flow of the parsing should be stored here.
  /// This should be small and cheap to clone (maybe just a bunch of integers or booleans).
  /// If a var only represents a resource (e.g. a chunk of memory, a channel, etc),
  /// it should be stored in [`Self::heap`].
  ///
  /// This is public. You can mutate this directly if needed.
  ///
  /// You can access this in [`Action`]s via [`Input::state`]
  /// and [`Accepted::state`](crate::combinator::Accepted::state).
  pub state: T::State,

  /// The reusable heap.
  ///
  /// With this, you can reuse allocated memory
  /// across actions and parsings.
  ///
  /// All vars that doesn't count as a part of [`Self::state`] should be stored here.
  /// If a var is used to control the flow of the parsing,
  /// it should be treated as a state and stored in [`Self::state`].
  /// If a var only represents a resource (e.g. a chunk of memory, a channel, etc),
  /// it should be stored here.
  ///
  /// This is public. You can mutate this directly if needed.
  ///
  /// You can access this in [`Action`]s via [`Input::heap`]
  /// and [`Accepted::heap`](crate::combinator::Accepted::heap).
  pub heap: T::Heap,

  /// The progress of the parser. You can mutate this directly if needed.
  ///
  /// See [`Instant`].
  pub instant: Instant<&'text T::Text>,

  /// The entry action.
  pub entry: T,
}

impl<'text, T: Action<State: Clone, Heap: Clone> + Clone> Clone for Parser<'text, T> {
  fn clone(&self) -> Self {
    Parser {
      state: self.state.clone(),
      heap: self.heap.clone(),
      instant: self.instant.clone(),
      entry: self.entry.clone(),
    }
  }
}

impl Parser<'static, Take> {
  /// Create a parser builder with default settings.
  #[inline]
  pub const fn builder() -> Builder<()> {
    Builder::new()
  }
}

impl<'text, T: Action> Parser<'text, T> {
  /// Consume self, return a new instance with the same action and a new text.
  ///
  /// [`Self::instant`] and [`Self::state`] will be reset to default.
  /// [`Self::heap`] won't change.
  #[inline]
  pub fn reload(self, text: &T::Text) -> Parser<T>
  where
    T::State: Default,
  {
    self.reload_with(T::State::default(), text)
  }

  /// Consume self, return a new instance with the same action, a new text and an optional new state.
  ///
  /// If the state is not provided, current [`Self::state`] will be kept.
  /// [`Self::instant`] will be reset to default.
  /// [`Self::heap`] won't change.
  #[inline]
  pub fn reload_with(self, state: impl Into<Option<T::State>>, text: &T::Text) -> Parser<T> {
    Parser {
      entry: self.entry,
      heap: self.heap,
      state: state.into().unwrap_or(self.state),
      instant: Instant::new(text),
    }
  }

  /// Take a snapshot of the current [`Self::state`] and [`Self::instant`].
  #[inline]
  pub fn snapshot(&self) -> Snapshot<&'text T::Text, T::State>
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
  pub fn restore(&mut self, snapshot: Snapshot<&'text T::Text, T::State>) {
    self.state = snapshot.state;
    self.instant = snapshot.instant;
  }

  /// Try to yield the next [`Output`] without updating [`Self::instant`] and [`Self::state`].
  /// [`Self::state`] will be cloned and returned.
  /// Return [`None`] if the action rejects.
  #[inline]
  pub fn peek(&mut self) -> (Option<Output<T::Value>>, T::State)
  where
    T::State: Clone,
  {
    let mut tmp_state = self.state.clone();
    (
      self.entry.exec(Input {
        instant: &self.instant,
        state: &mut tmp_state,
        heap: &mut self.heap,
      }),
      tmp_state,
    )
  }
}

impl<'text, T: Action<Text: Digest>> Iterator for Parser<'text, T>
where
  RangeFrom<usize>: SliceIndex<T::Text, Output = T::Text>,
{
  type Item = Output<T::Value>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self
      .entry
      .exec(Input {
        instant: &self.instant,
        state: &mut self.state,
        heap: &mut self.heap,
      })
      .inspect(|output| unsafe { self.instant.digest_unchecked(output.digested) })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::contextual;
  use std::rc::Rc;

  #[test]
  fn parser_builder() {
    contextual!(i32, i32);

    let parser = Parser::builder()
      .state(123)
      .heap(123)
      .entry(eat("123"))
      .build("123");
    assert_eq!(parser.state, 123);
    assert_eq!(parser.heap, 123);
    assert_eq!(parser.instant.text(), "123");
  }

  #[test]
  fn parser_clone() {
    contextual!(i32, i32);

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
    contextual!(i32, i32);

    let parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    assert_eq!(
      parser
        .entry
        .exec(Input {
          instant: &Instant::new("123"),
          state: &mut 0,
          heap: &mut 0
        })
        .unwrap()
        .digested,
      3
    );
    assert_eq!(parser.instant.digested(), 0);
  }

  #[test]
  fn parser_reload() {
    contextual!(i32, i32);

    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    parser.next();
    assert_eq!(parser.instant.digested(), 3);
    assert_eq!(parser.instant.rest(), "");
    let parser = parser.reload("456");
    assert_eq!(parser.instant.text(), "456");
    assert_eq!(parser.instant.rest(), "456");
    assert_eq!(parser.instant.digested(), 0);
    assert_eq!(parser.state, 0);
    assert_eq!(parser.heap, 123);
  }

  #[test]
  fn parser_reload_with() {
    contextual!(i32, i32);

    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    parser.next();
    assert_eq!(parser.instant.digested(), 3);
    assert_eq!(parser.instant.rest(), "");
    let parser = parser.reload_with(None, "456");
    assert_eq!(parser.instant.text(), "456");
    assert_eq!(parser.instant.rest(), "456");
    assert_eq!(parser.instant.digested(), 0);
    assert_eq!(parser.state, 123);
    assert_eq!(parser.heap, 123);
  }

  #[test]
  fn parser_snapshot_restore() {
    contextual!(i32, i32);

    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    parser.next();
    let snapshot = parser.snapshot();
    assert_eq!(snapshot.state, 123);
    assert_eq!(snapshot.instant.text(), "123");
    assert_eq!(snapshot.instant.digested(), 3);
    assert_eq!(snapshot.instant.rest(), "");

    let mut parser = Parser {
      state: 0,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    parser.restore(snapshot);
    assert_eq!(parser.state, 123);
    assert_eq!(parser.instant.text(), "123");
    assert_eq!(parser.instant.digested(), 3);
    assert_eq!(parser.instant.rest(), "");
  }

  #[test]
  fn parser_parse() {
    contextual!(i32, i32);

    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123"),
      entry: eat("123"),
    };
    let output = parser.next().unwrap();
    assert_eq!(output.digested, 3);
    assert_eq!(output.value, ());
    assert_eq!(parser.instant.digested(), 3);
    assert_eq!(parser.instant.rest(), "");
    assert!(parser.next().is_none());
  }

  #[test]
  fn parser_peek() {
    contextual!(i32, i32);

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
    assert_eq!(parser.instant.digested(), 0);
    assert_eq!(parser.instant.rest(), "123");
    assert!(parser.next().is_some());
  }

  #[test]
  fn parser_iterator_in_for_loop() {
    contextual!(i32, i32);

    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123123123"),
      entry: eat("123"),
    };
    for o in &mut parser {
      assert_eq!(o.digested, 3);
    }
    assert_eq!(parser.instant.digested(), 9);
  }

  #[test]
  fn parser_iterator_with_iter_methods() {
    contextual!(i32, i32);

    let mut parser = Parser {
      state: 123,
      heap: 123,
      instant: Instant::new("123123123"),
      entry: eat("123"),
    };
    for (_, o) in (&mut parser).enumerate() {
      assert_eq!(o.digested, 3);
    }
    assert_eq!(parser.instant.digested(), 9);
  }
}
