#[derive(Clone)]
pub struct LexerState<'text> {
  text: &'text str,
  digested: usize,
  trimmed: bool,
}

impl<'text> LexerState<'text> {
  pub fn new(text: &'text str) -> Self {
    LexerState {
      text,
      digested: 0,
      trimmed: text.len() == 0, // if text is empty, no need to trim
    }
  }

  pub fn text(&self) -> &'text str {
    self.text
  }
  pub fn digested(&self) -> usize {
    self.digested
  }
  pub fn trimmed(&self) -> bool {
    self.trimmed
  }

  pub fn rest(&self) -> &'text str {
    &self.text[self.digested..]
  }

  pub fn digest(&mut self, n: usize) {
    if n == 0 {
      return;
    }

    // update other states
    self.digested += n;
    self.trimmed = self.digested == self.text.len(); // if all chars are digested, no need to trim
  }

  pub fn trim(&mut self, digested: usize) {
    self.digested += digested;
    self.trimmed = true;
  }
}
