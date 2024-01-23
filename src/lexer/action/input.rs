pub struct ActionInput<'buffer, 'state, ActionState> {
  buffer: &'buffer str,
  start: usize,
  state: &'state mut ActionState,
  peek: bool,
}

impl<'buffer, 'state, ActionState> ActionInput<'buffer, 'state, ActionState> {
  pub fn new(
    buffer: &'buffer str,
    start: usize,
    state: &'state mut ActionState,
    peek: bool,
  ) -> Self {
    ActionInput {
      buffer,
      start,
      state,
      peek,
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

  pub fn state(&self) -> &ActionState {
    self.state
  }
  pub fn state_mut(&mut self) -> &mut ActionState {
    self.state
  }

  /// Whether this evaluation is a peek.
  /// If `true`, you may NOT want to mutate the action state.
  pub fn peek(&self) -> bool {
    self.peek
  }

  /// The rest of the input text.
  /// This is a shortcut for `&self.buffer[self.start..]`.
  pub fn rest(&self) -> &'buffer str {
    &self.buffer[self.start..]
  }
}
