use super::Parser;
use crate::{combinator, parse::Parse};

/// A builder for [`Parser`].
pub struct Builder<Entry, State, Heap> {
  state: State,
  heap: Heap,
  entry: Entry,
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

impl<Entry, State, Heap> Builder<Entry, State, Heap> {
  /// Set [`Parser::state`].
  pub fn state<NewState>(self, state: NewState) -> Builder<Entry, NewState, Heap> {
    Builder {
      state,
      heap: self.heap,
      entry: self.entry,
    }
  }

  /// Set [`Parser::heap`].
  pub fn heap<NewHeap>(self, heap: NewHeap) -> Builder<Entry, State, NewHeap> {
    Builder {
      heap,
      state: self.state,
      entry: self.entry,
    }
  }

  /// Set [`Parser::entry`].
  pub fn entry<'a, R: Parse<State, Heap> + 'a>(
    self,
    builder: impl FnOnce(combinator::Builder<State, Heap>) -> R + 'a,
  ) -> Builder<Box<dyn Parse<State, Heap, Kind = R::Kind> + 'a>, State, Heap> {
    Builder {
      entry: Box::new(builder(combinator::Builder::new())),
      state: self.state,
      heap: self.heap,
    }
  }
}

impl<'a, Kind, State, Heap> Builder<Box<dyn Parse<State, Heap, Kind = Kind> + 'a>, State, Heap> {
  /// Build a [`Parser`] with the given text.
  pub fn build<'text>(self, text: &'text str) -> Parser<'a, 'text, Kind, State, Heap> {
    Parser {
      state: self.state,
      heap: self.heap,
      text,
      rest: text,
      entry: self.entry,
    }
  }
}
