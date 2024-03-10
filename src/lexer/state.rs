#[derive(Clone)]
pub struct LexerState<'text> {
  text: &'text str,
  digested: usize,
}

impl<'text> LexerState<'text> {
  pub fn new(text: &'text str) -> Self {
    LexerState { text, digested: 0 }
  }

  pub fn text(&self) -> &'text str {
    self.text
  }
  pub fn digested(&self) -> usize {
    self.digested
  }

  pub fn rest(&self) -> &'text str {
    &self.text[self.digested..]
  }

  pub fn digest(&mut self, n: usize) {
    self.digested += n;
  }
}
