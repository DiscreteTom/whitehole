pub struct ActionInput<'state, ActionState> {
  /// The whole input text.
  buffer: &'static str,
  /// From where to lex.
  start: usize,
  // TODO: add comment
  state: &'state mut ActionState,
  // TODO: add peek/rest
}

impl<'state, ActionState> ActionInput<'state, ActionState> {
  pub fn new(buffer: &'static str, start: usize, state: &'state mut ActionState) -> Self {
    ActionInput {
      buffer,
      start,
      state,
    }
  }

  pub fn buffer(&self) -> &'static str {
    self.buffer
  }

  pub fn start(&self) -> usize {
    self.start
  }

  pub fn state(&mut self) -> &mut ActionState {
    self.state
  }

  pub fn rest(&self) -> &'static str {
    &self.buffer[self.start..]
  }
}
