#[derive(Clone)]
pub struct LexerState<'buffer, ActionState: Clone + Default> {
  buffer: &'buffer str,
  digested: usize,
  trimmed: bool,
  action_state: ActionState,
}

impl<'buffer, ActionState: Clone + Default> LexerState<'buffer, ActionState> {
  pub fn new(buffer: &'buffer str) -> Self {
    LexerState {
      buffer,
      digested: 0,
      trimmed: buffer.len() == 0, // if buffer is empty, no need to trim
      action_state: ActionState::default(),
    }
  }

  pub fn buffer(&self) -> &'buffer str {
    self.buffer
  }
  pub fn digested(&self) -> usize {
    self.digested
  }
  pub fn trimmed(&self) -> bool {
    self.trimmed
  }
  pub fn action_state(&self) -> &ActionState {
    &self.action_state
  }
  pub fn action_state_mut(&mut self) -> &mut ActionState {
    &mut self.action_state
  }

  pub fn digest(&mut self, n: usize) {
    if n == 0 {
      return;
    }

    // update other states
    self.digested += n;
    self.trimmed = self.digested == self.buffer.len(); // if all chars are digested, no need to trim
  }

  pub fn trim(&mut self, digested: usize) {
    self.digested += digested;
    self.trimmed = true;
  }
}
