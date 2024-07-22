pub struct ActionInput<'text, 'action_state, ActionState> {
  // users can mutate the action state directly, so it's public.
  // with the action state, users can build stateful lexers,
  // while actions remain stateless and clone-able.
  pub state: &'action_state mut ActionState,

  // below fields are readonly

  // users could access the whole input text instead of only the rest of text,
  // to check chars before the start position if needed
  /// See [`Self::text`].
  text: &'text str,
  /// See [`Self::start`].
  start: usize,
  // cache the rest of the text to prevent creating the slice every time
  // because `input.rest` is frequently used across all actions during the lexing loop.
  /// See [`Self::rest`].
  rest: &'text str,
  // this is precalculated and cached because this might be used for at least once
  // when traversing string body matchers
  /// See [`Self::next`].
  next: char,
}

impl<'text, 'action_state, ActionState> ActionInput<'text, 'action_state, ActionState> {
  /// Return [`None`] if [`start`](Self::start) is equal to the length of
  /// [`text`](Self::text).
  /// # Panics
  /// This method panics if [`start`](Self::start) is out of bounds of
  /// [`text`](Self::text).
  pub fn new(
    text: &'text str,
    start: usize,
    state: &'action_state mut ActionState,
  ) -> Option<Self> {
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
  pub fn text(&self) -> &'text str {
    self.text
  }

  /// From where to lex, in bytes.
  /// This is guaranteed to be smaller than the length of [`Self::text`].
  pub fn start(&self) -> usize {
    self.start
  }

  /// The undigested part of the input text.
  /// When lexing this is guaranteed to be not empty.
  pub fn rest(&self) -> &'text str {
    self.rest
  }

  /// The next char in the rest of the input text.
  pub fn next(&self) -> char {
    self.next
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
    assert!(ActionInput::new("123", 4, &mut state).is_none());
  }
}
