#[derive(Debug)]
pub struct ActionInput<'text, ActionStateRef> {
  /// An reference of the `ActionState`.
  ///
  /// This is public, so if this is `&mut ActionState` then you can mutate this directly.
  ///
  /// With the `ActionState`, you can construct stateful lexers,
  /// while actions remain stateless and clone-able.
  pub state: ActionStateRef,

  /// See [`Self::text`].
  text: &'text str,
  /// See [`Self::start`].
  start: usize,
  /// See [`Self::rest`].
  rest: &'text str,
  /// See [`Self::next`].
  next: char,
}

impl<'text, ActionStateRef> ActionInput<'text, ActionStateRef> {
  /// Return [`None`] if the [`start`](Self::start) is equal to the length of
  /// [`text`](Self::text).
  /// # Panics
  /// This method panics if the [`start`](Self::start) is out of bounds of
  /// [`text`](Self::text).
  pub fn new(text: &'text str, start: usize, state: ActionStateRef) -> Option<Self> {
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

  /// Consume self, return a new instance with a new `ActionStateRef`.
  #[inline]
  pub(crate) fn reload<NewActionStateRef>(
    self,
    s: NewActionStateRef,
  ) -> ActionInput<'text, NewActionStateRef> {
    ActionInput {
      state: s,
      text: self.text,
      start: self.start,
      rest: self.rest,
      next: self.next,
    }
  }
}

impl<'text, 'action_state, ActionState> ActionInput<'text, &'action_state mut ActionState> {
  /// Cast `ActionInput<&mut ActionState>` to `ActionInput<&ActionState>`
  #[inline]
  pub fn as_ref<'input>(&'input self) -> ActionInput<'text, &'input ActionState> {
    // TODO(perf): maybe just transmute self?
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
}
