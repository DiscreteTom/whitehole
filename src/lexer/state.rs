#[derive(Clone)]
pub struct LexerState {
  buffer: String,
  digested: usize,
  trimmed: bool,
}

impl Default for LexerState {
  fn default() -> Self {
    LexerState {
      buffer: String::default(),
      digested: 0,
      trimmed: true, // no input, so it's trimmed
    }
  }
}

impl LexerState {
  pub fn reset(&mut self) {
    self.buffer.clear();
    self.digested = 0;
    self.trimmed = true; // no input, so it's trimmed
  }

  pub fn feed(&mut self, s: &str) {
    if s.len() == 0 {
      return;
    }

    self.buffer += s;
    self.trimmed = false; // maybe the new feed chars can make muted actions accept
  }

  pub fn digest(&mut self, n: usize) {
    if n == 0 {
      return;
    }

    // update other states
    self.digested += n;
    self.trimmed = self.digested == self.buffer.len(); // if all chars are digested, no need to trim
  }

  pub fn trim(&mut self) {
    self.trimmed = true;
  }

  pub fn rest(&self) -> &str {
    &self.buffer[self.digested..]
  }
}
