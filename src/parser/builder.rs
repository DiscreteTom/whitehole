use super::Parser;
use crate::parse::Parse;

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
  pub fn entry<'a, Kind>(
    self,
    entry: impl Parse<Kind = Kind, State = State, Heap = Heap> + 'a,
  ) -> Builder<Box<dyn Parse<Kind = Kind, State = State, Heap = Heap> + 'a>, State, Heap> {
    Builder {
      entry: Box::new(entry),
      state: self.state,
      heap: self.heap,
    }
  }
}

impl<'a, Kind, State, Heap>
  Builder<Box<dyn Parse<Kind = Kind, State = State, Heap = Heap> + 'a>, State, Heap>
{
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
