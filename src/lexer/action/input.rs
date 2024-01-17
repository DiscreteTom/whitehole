pub struct ActionInput<'buffer, 'state, ActionState> {
  /// The whole input text.
  buffer: &'buffer str,
  /// From where to lex.
  start: usize,
  // TODO: add comment
  state: &'state mut ActionState,
  // TODO: add peek/rest
}

impl<'buffer, 'state, ActionState> ActionInput<'buffer, 'state, ActionState> {
  pub fn new(buffer: &'buffer str, start: usize, state: &'state mut ActionState) -> Self {
    ActionInput {
      buffer,
      start,
      state,
    }
  }

  pub fn buffer(&self) -> &'buffer str {
    self.buffer
  }

  pub fn start(&self) -> usize {
    self.start
  }

  pub fn state(&mut self) -> &mut ActionState {
    self.state
  }

  pub fn rest(&self) -> &'buffer str {
    &self.buffer[self.start..]
  }
}
