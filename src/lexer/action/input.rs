use crate::lexer::token::Range;

#[derive(Debug)]
pub struct ActionInput<'text, StateRef, HeapRef> {
  /// This is often `&mut State`.
  /// This is public, so you can mutate the `State` directly.
  ///
  /// With the `State`, you can construct stateful lexers,
  /// while actions remain stateless and clone-able.
  ///
  /// All vars that control the flow of the lexing loop should be stored here.
  /// This should be small and cheap to clone (maybe just a bunch of integers or booleans).
  /// If a var only represents a resource (e.g. a chunk of memory, a channel, etc),
  /// it should be stored in [`Self::heap`].
  pub state: StateRef,
  /// This is often `&mut Heap`.
  /// This is public, so you can mutate this directly.
  ///
  /// With the `Heap`, you can re-use allocated memory
  /// across actions and lexing loops.
  ///
  /// All vars that doesn't count as a part of [`Self::state`] should be stored here.
  /// If a var is used to control the flow of the lexing loop,
  /// it should be treated as a state and stored in [`Self::state`].
  /// If a var only represents a resource (e.g. a chunk of memory, a channel, etc),
  /// it should be stored here.
  pub heap: HeapRef,

  /// See [`Self::text`].
  text: &'text str,
  /// See [`Self::start`].
  start: usize,
  /// See [`Self::rest`].
  rest: &'text str,
  /// See [`Self::next`].
  next: char,
}

impl<'text, StateRef, HeapRef> ActionInput<'text, StateRef, HeapRef> {
  /// Return [`None`] if the [`start`](Self::start) is equal to the length of
  /// [`text`](Self::text).
  /// # Panics
  /// This method panics if the [`start`](Self::start) is out of bounds of
  /// [`text`](Self::text).
  #[inline]
  pub fn new(text: &'text str, start: usize, state: StateRef, heap: HeapRef) -> Option<Self> {
    let rest = &text[start..];

    rest.chars().next().map(|next| Self {
      text,
      start,
      rest,
      next,
      state,
      heap,
    })
  }

  /// The whole input text.
  ///
  /// You can access the whole input text instead of only the rest of text,
  /// so that you can check chars before the [`Self::start`] position if needed.
  #[inline]
  pub const fn text(&self) -> &'text str {
    self.text
  }

  /// From where to lex, in bytes.
  /// This is guaranteed to be smaller than the length of [`Self::text`].
  #[inline]
  pub const fn start(&self) -> usize {
    self.start
  }

  /// The undigested part of the input text.
  /// This is guaranteed to be not empty.
  ///
  /// This is precalculated and cached to prevent creating the slice every time
  /// because this is frequently used across all actions during the lexing loop.
  #[inline]
  pub const fn rest(&self) -> &'text str {
    self.rest
  }

  /// The next char in the rest of the input text.
  ///
  /// This is precalculated and cached because this will be used for at least once
  /// when traversing actions in the lexing loop.
  #[inline]
  pub const fn next(&self) -> char {
    self.next
  }

  /// A helper method to create a [`Range`] from [`Self::start`](Self::start) and
  /// the `digested` length.
  /// # Safety
  /// This method won't check if the `digested` length is
  /// greater than the length of [`Self::rest`].
  /// For a safer version, use [`Self::range`].
  #[inline]
  pub const fn range_unchecked(&self, digested: usize) -> Range {
    Range {
      start: self.start,
      end: self.start + digested,
    }
  }

  /// A helper method to create a [`Range`] from [`Self::start`](Self::start) and
  /// the `digested` length.
  ///
  /// Return [`None`] if the `digested` length is greater than the length of [`Self::rest`].
  #[inline]
  pub const fn range(&self, digested: usize) -> Option<Range> {
    if digested > self.rest.len() {
      None
    } else {
      Some(self.range_unchecked(digested))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn action_input_at_start() {
    let mut state = ();
    let mut heap = ();
    let input = ActionInput::new("123", 0, &mut state, &mut heap).unwrap();
    assert_eq!(input.text(), "123");
    assert_eq!(input.start(), 0);
    assert_eq!(input.rest(), "123");
    assert_eq!(input.next(), '1');
  }

  #[test]
  fn action_input_in_the_middle() {
    let mut state = ();
    let mut heap = ();
    let input = ActionInput::new("123", 1, &mut state, &mut heap).unwrap();
    assert_eq!(input.text(), "123");
    assert_eq!(input.start(), 1);
    assert_eq!(input.rest(), "23");
    assert_eq!(input.next(), '2');
  }

  #[test]
  fn action_input_no_rest() {
    let mut state = ();
    assert!(ActionInput::new("123", 3, &mut state, &mut ()).is_none());
  }

  #[test]
  #[should_panic]
  fn action_input_out_of_text() {
    let mut state = ();
    ActionInput::new("123", 4, &mut state, &mut ());
  }

  #[test]
  fn action_input_range_unchecked() {
    let mut state = ();
    let mut heap = ();
    let input = ActionInput::new("123", 1, &mut state, &mut heap).unwrap();
    assert_eq!(input.range_unchecked(2), Range { start: 1, end: 3 });
    assert_eq!(input.range_unchecked(3), Range { start: 1, end: 4 });
  }

  #[test]
  fn action_input_range() {
    let mut state = ();
    let mut heap = ();
    let input = ActionInput::new("123", 1, &mut state, &mut heap).unwrap();
    assert_eq!(input.range(2), Some(Range { start: 1, end: 3 }));
    assert_eq!(input.range(3), None);
  }
}
