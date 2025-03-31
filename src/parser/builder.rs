use super::{Instant, Parser};
use crate::action::Action;

/// A builder for [`Parser`].
/// # Examples
/// ## Basics
/// ```
/// use whitehole::{parser::Parser, combinator::eat};
///
/// Parser::builder()
///   // Set the entry action
///   .entry(eat("hello ") + "world")
///   // Build the parser
///   .build("hello world");
/// ```
/// ## Contextual
/// ```
/// use whitehole::{parser::Parser, combinator::contextual};
///
/// # struct MyState;
/// # impl MyState {
/// #   fn new() -> Self { MyState }
/// # }
/// # struct MyHeap;
/// # impl MyHeap {
/// #   fn new() -> Self { MyHeap }
/// # }
/// // Generate contextual combinators
/// contextual!(MyState, MyHeap);
///
/// Parser::builder()
///   // Set the state
///   .state(MyState::new())
///   // Set the heap
///   .heap(MyHeap::new())
///   // Set the entry action
///   .entry(eat("hello ") + "world")
///   // Build the parser
///   .build("hello world");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Builder<T, State = (), Heap = ()> {
  state: State,
  heap: Heap,
  entry: T,
}

impl Builder<(), (), ()> {
  /// Create a new instance with [`Parser::state`] and [`Parser::heap`] set to `()`.
  #[inline]
  pub const fn new() -> Self {
    Builder {
      state: (),
      heap: (),
      entry: (),
    }
  }
}

impl Default for Builder<(), (), ()> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<T, State, Heap> Builder<T, State, Heap> {
  /// Set [`Parser::state`].
  #[inline]
  pub fn state<NewState>(self, state: NewState) -> Builder<T, NewState, Heap> {
    Builder {
      state,
      heap: self.heap,
      entry: self.entry,
    }
  }

  /// Set [`Parser::heap`].
  #[inline]
  pub fn heap<NewHeap>(self, heap: NewHeap) -> Builder<T, State, NewHeap> {
    Builder {
      heap,
      state: self.state,
      entry: self.entry,
    }
  }

  /// Set [`Parser::entry`].
  #[inline]
  pub fn entry<Entry>(self, entry: Entry) -> Builder<Entry, State, Heap> {
    Builder {
      entry,
      state: self.state,
      heap: self.heap,
    }
  }

  /// Build a [`Parser`] with the given text.
  #[inline]
  pub fn build<Text: ?Sized>(self, text: &Text) -> Parser<T>
  where
    T: Action<Text = Text, State = State, Heap = Heap>,
  {
    Parser {
      state: self.state,
      heap: self.heap,
      entry: self.entry,
      instant: Instant::new(text),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::eat, contextual};

  #[test]
  fn parser_builder_default() {
    let mut parser = Builder::default()
      .entry(eat("hello ") + "world")
      .build("hello world");

    let output = parser.next().unwrap();
    assert_eq!(output.digested, 11);
    let _: () = output.value;

    assert!(parser.next().is_none());

    let _: () = parser.heap;
    let _: () = parser.state;
  }

  #[test]
  fn parser_builder_with_state_heap() {
    contextual!(i32, i32);

    let mut parser = Builder::default()
      .state(1)
      .heap(1)
      .entry((eat("hello ") + "world").then(|input| {
        *input.state = 1;
        *input.heap = 1;
      }))
      .build("hello world");

    let output = parser.next().unwrap();
    assert_eq!(output.digested, 11);
    let _: () = output.value;

    assert!(parser.next().is_none());

    assert_eq!(parser.heap, 1);
    assert_eq!(parser.state, 1);
  }

  #[test]
  fn re_use_entry_with_ref() {
    let entry = eat("hello ") + "world";

    let mut p1 = Builder::default().entry(&entry).build("hello world");
    let mut p2 = Builder::default().entry(&entry).build("hello");

    assert!(p1.next().is_some());
    assert!(p2.next().is_none());
  }
}
