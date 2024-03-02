pub struct ActionInput<'buffer, 'state, ActionState> {
  buffer: &'buffer str,
  start: usize,
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
