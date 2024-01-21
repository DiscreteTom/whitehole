#[derive(Clone)]
pub struct LexerState {
  // when `lexer.lex` we want to change state but not the buffer
  // so we have to separate mutable states and buffer in different structs
  digested: usize,
  trimmed: bool,
}

impl Default for LexerState {
  fn default() -> Self {
    LexerState {
      digested: 0,
      trimmed: true, // no input, so it's trimmed
    }
  }
}

impl LexerState {
  pub fn digested(&self) -> usize {
    self.digested
  }
  pub fn trimmed(&self) -> bool {
    self.trimmed
  }

  pub fn reset(&mut self) {
    self.digested = 0;
    self.trimmed = true; // no input, so it's trimmed
  }

  // TODO: better name?
  pub fn on_feed(&mut self) {
    self.trimmed = false; // maybe the new feed chars can make muted actions accept
  }

  pub fn on_digest(&mut self, n: usize, buffer: &str) {
    if n == 0 {
      return;
    }

    // update other states
    self.digested += n;
    self.trimmed = self.digested == buffer.len(); // if all chars are digested, no need to trim
  }
}
