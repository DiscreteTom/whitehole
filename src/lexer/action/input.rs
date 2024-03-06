pub struct ActionInput<'text, ActionState> {
  // we store the whole text instead of only storing the rest of text
  // so that users can check chars before the start position if needed
  text: &'text str,
  start: usize,
  // cache the rest of the text
  // to prevent create the slice every time
  rest: &'text str,
  // user can mutate the action state
  pub state: ActionState,
}

impl<'text, ActionState> ActionInput<'text, ActionState> {
  pub fn new(text: &'text str, start: usize, state: ActionState) -> Self {
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

  /// From where to lex.
  pub fn start(&self) -> usize {
    self.start
  }

  /// The undigested part of the input text.
  pub fn rest(&self) -> &'text str {
    self.rest
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn action_input() {
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
