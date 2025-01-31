use super::{Instant, Parser};
use crate::action::Action;

/// A builder for [`Parser`].
/// # Examples
/// ```
/// use whitehole::{parser::Parser, combinator::eat};
///
/// # struct MyState;
/// # impl MyState {
/// #   fn new() -> Self { MyState }
/// # }
/// # struct MyHeap;
/// # impl MyHeap {
/// #   fn new() -> Self { MyHeap }
/// # }
/// Parser::builder()
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
  pub fn entry<Text: ?Sized, Entry: Action<Text, State, Heap>>(
    self,
    entry: Entry,
  ) -> Builder<Entry, State, Heap> {
    Builder {
      entry,
      state: self.state,
      heap: self.heap,
    }
  }

  /// Build a [`Parser`] with the given text.
  #[inline]
  pub fn build<Text: ?Sized>(self, text: &Text) -> Parser<T, &Text, State, Heap>
  where
    T: Action<Text, State, Heap>,
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
  use crate::combinator::eat;

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
    let mut parser = Builder::default()
      .state(1)
      .heap(1)
      .entry((eat("hello ") + "world").then(|mut ctx| {
        *ctx.state() = 1;
        *ctx.heap() = 1;
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

  #[test]
  fn str_in_heap() {
    let text = "123".to_string();
    let mut parser = Builder::new()
      .heap(&text)
      .entry(eat(text.as_str()))
      .build(text.as_str());
    assert!(parser.next().is_some());
  }
}
