pub struct ActionInput<ActionState: 'static> {
  /// The whole input text.
  pub buffer: &'static str,
  /// From where to lex.
  pub start: usize,
  // TODO: add comment
  pub state: &'static ActionState,
  // TODO: add peek/rest
}

impl<ActionState> ActionInput<ActionState> {
  pub fn rest(&self) -> &'static str {
    &self.buffer[self.start..]
  }
}
