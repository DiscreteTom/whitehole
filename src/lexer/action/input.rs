pub struct ActionInput<'buffer, 'state, ActionState> {
  buffer: &'buffer str,
  start: usize,
  // user can mutate the action state
  pub state: &'state mut ActionState,
}

impl<'buffer, 'state, ActionState> ActionInput<'buffer, 'state, ActionState> {
  pub fn new(buffer: &'buffer str, start: usize, state: &'state mut ActionState) -> Self {
    ActionInput {
      buffer,
      start,
      state,
    }
  }

  /// The whole input text.
  pub fn buffer(&self) -> &'buffer str {
    self.buffer
  }

  /// From where to lex.
  pub fn start(&self) -> usize {
    self.start
  }

  /// The rest of the input text.
  /// This is a shortcut for `&self.buffer[self.start..]`.
  pub fn rest(&self) -> &'buffer str {
    &self.buffer[self.start..]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn action_input() {
    let mut state = ();
    let input = ActionInput::new("123", 1, &mut state);
    assert_eq!(input.buffer(), "123");
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
