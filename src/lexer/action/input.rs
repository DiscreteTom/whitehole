#[derive(Debug)]
pub struct ActionInput<'text, StateRef> {
  /// An reference of the `State`.
  ///
  /// This is public, so if this is `&mut State` then you can mutate this directly.
  ///
  /// With the `State`, you can construct stateful lexers,
  /// while actions remain stateless and clone-able.
  pub state: StateRef,

  /// See [`Self::text`].
  text: &'text str,
  /// See [`Self::start`].
  start: usize,
  /// See [`Self::rest`].
  rest: &'text str,
  /// See [`Self::next`].
  next: char,
}

impl<'text, StateRef> ActionInput<'text, StateRef> {
  /// Return [`None`] if the [`start`](Self::start) is equal to the length of
  /// [`text`](Self::text).
  /// # Panics
  /// This method panics if the [`start`](Self::start) is out of bounds of
  /// [`text`](Self::text).
  pub fn new(text: &'text str, start: usize, state: StateRef) -> Option<Self> {
    let rest = &text[start..];

    rest.chars().next().map(|next| Self {
      text,
      start,
      rest,
      next,
      state,
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

  /// Consume self, return a new instance with a new `StateRef`.
  #[inline]
  pub(crate) fn reload<NewStateRef>(self, s: NewStateRef) -> ActionInput<'text, NewStateRef> {
    ActionInput {
      state: s,
      text: self.text,
      start: self.start,
      rest: self.rest,
      next: self.next,
    }
  }
}

impl<'text, 'state, State> ActionInput<'text, &'state mut State> {
  /// Cast `ActionInput<&mut State>` to `ActionInput<&State>`
  #[inline]
  pub fn as_ref<'this>(&'this self) -> ActionInput<'text, &'this State> {
    ActionInput {
      state: self.state,
      text: self.text,
      start: self.start,
      rest: self.rest,
      next: self.next,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn action_input_at_start() {
    let mut state = ();
    let input = ActionInput::new("123", 0, &mut state).unwrap();
    assert_eq!(input.text(), "123");
    assert_eq!(input.start(), 0);
    assert_eq!(input.rest(), "123");
    assert_eq!(input.next(), '1');
  }

  #[test]
  fn action_input_in_the_middle() {
    let mut state = ();
    let input = ActionInput::new("123", 1, &mut state).unwrap();
    assert_eq!(input.text(), "123");
    assert_eq!(input.start(), 1);
    assert_eq!(input.rest(), "23");
    assert_eq!(input.next(), '2');
  }

  #[test]
  fn action_input_no_rest() {
    let mut state = ();
    assert!(ActionInput::new("123", 3, &mut state).is_none());
  }

  #[test]
  #[should_panic]
  fn action_input_out_of_text() {
    let mut state = ();
    ActionInput::new("123", 4, &mut state);
  }

  #[test]
  fn action_input_reload() {
    let mut state = ();
    let input = ActionInput::new("123", 1, &mut state).unwrap();
    let mut state = 123;
    let input = input.reload(&mut state);
    assert_eq!(input.text(), "123");
    assert_eq!(input.start(), 1);
    assert_eq!(input.rest(), "23");
    assert_eq!(input.next(), '2');
  }
}
