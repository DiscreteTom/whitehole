use super::{Instant, Parser};
use crate::{combinator, parse::Parse};

/// A builder for [`Parser`].
/// # Examples
/// ```
/// use whitehole::parser::Builder;
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
///   // build the entry combinator with the provided state and heap
///   .entry(|b| b.eat("hello ") + "world")
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
  pub const fn new() -> Self {
    Builder {
      state: (),
      heap: (),
      entry: (),
    }
  }
}

impl Default for Builder<(), (), ()> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T, State, Heap> Builder<T, State, Heap> {
  /// Set [`Parser::state`].
  pub fn state<NewState>(self, state: NewState) -> Builder<T, NewState, Heap> {
    Builder {
      state,
      heap: self.heap,
      entry: self.entry,
    }
  }

  /// Set [`Parser::heap`].
  pub fn heap<NewHeap>(self, heap: NewHeap) -> Builder<T, State, NewHeap> {
    Builder {
      heap,
      state: self.state,
      entry: self.entry,
    }
  }

  /// Set [`Parser::entry`].
  pub fn entry<R: Parse<State, Heap>>(
    self,
    builder: impl FnOnce(combinator::Builder<State, Heap>) -> R,
  ) -> Builder<R, State, Heap> {
    Builder {
      entry: builder(combinator::Builder::new()),
      state: self.state,
      heap: self.heap,
    }
  }
}

impl<T: Parse<State, Heap>, State, Heap> Builder<T, State, Heap> {
  /// Build a [`Parser`] with the given text.
  pub fn build(self, text: &str) -> Parser<T, State, Heap> {
    Parser {
      state: self.state,
      heap: self.heap,
      entry: self.entry,
      instant: Instant::new(text),
    }
  }
}

// TODO: tests
