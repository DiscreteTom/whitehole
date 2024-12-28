use super::{Instant, Parser};
use crate::action::Action;

/// A builder for [`Parser`].
/// # Examples
/// ```
/// use whitehole::{parser::Builder, combinator::eat};
///
/// # struct MyState;
/// # impl MyState {
/// #   fn new() -> Self { MyState }
/// # }
/// # struct MyHeap;
/// # impl MyHeap {
/// #   fn new() -> Self { MyHeap }
/// # }
/// let _ = Builder::new()
///   // optional
///   .state(MyState::new())
///   // optional
///   .heap(MyHeap::new())
///   // set the entry action
///   .entry(eat("hello ") + "world")
///   // build the parser
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
  pub fn entry<Entry: Action<State = State, Heap = Heap>>(
    self,
    entry: Entry,
  ) -> Builder<Entry, State, Heap> {
    Builder {
      entry,
      state: self.state,
      heap: self.heap,
    }
  }
}

impl<T: Action<State = State, Heap = Heap>, State, Heap> Builder<T, State, Heap> {
  /// Build a [`Parser`] with the given text.
  #[inline]
  pub fn build(self, text: &str) -> Parser<T> {
    Parser {
      state: self.state,
      heap: self.heap,
      entry: self.entry,
      instant: Instant::new(text),
    }
  }
}

// TODO: tests
