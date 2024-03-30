pub struct ActionInput<'text, ActionState> {
  // users can mutate the action state directly, so it's public.
  // with the action state, users can build stateful lexer,
  // while actions remain stateless and clone-able.
  pub state: ActionState,
  // users could access the whole input text instead of only the rest of text,
  // to check chars before the start position if needed
  /// See [`Self::text`].
  text: &'text str,
  /// See [`Self::start`].
  start: usize,
  // cache the rest of the text to prevent create the slice every time
  // because `input.rest` is frequently used across all actions
  /// See [`Self::rest`].
  rest: &'text str,
}

impl<'text, ActionState> ActionInput<'text, ActionState> {
  pub fn new(text: &'text str, start: usize, state: ActionState) -> Self {
    if start >= text.len() {
      panic!("Invalid start position. This should be a bug, please report it at https://github.com/DiscreteTom/whitehole/issues/new");
    }

    ActionInput {
      text,
      start,
      state,
      rest: &text[start..],
    }
  }

  /// The whole input text.
  pub fn text(&self) -> &'text str {
    self.text
  }

  /// From where to lex, in bytes.
  pub fn start(&self) -> usize {
    self.start
  }

  /// The undigested part of the input text.
  /// When lexing this is guaranteed to be not empty.
  pub fn rest(&self) -> &'text str {
    self.rest
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn action_input_at_start() {
    let mut state = ();
    let input = ActionInput::new("123", 0, &mut state);
    assert_eq!(input.text(), "123");
    assert_eq!(input.start(), 0);
    assert_eq!(input.rest(), "123");
  }

  #[test]
  fn action_input_in_the_middle() {
    let mut state = ();
    let input = ActionInput::new("123", 1, &mut state);
    assert_eq!(input.text(), "123");
    assert_eq!(input.start(), 1);
    assert_eq!(input.rest(), "23");
  }

  #[test]
  fn action_input_no_rest() {
    let mut state = ();
    let input = ActionInput::new("123", 3, &mut state);
    assert_eq!(input.rest(), "");
  }
}
