#[derive(Clone)]
pub struct LexerState<'text> {
  text: &'text str,
  digested: usize,
  // we don't store error tokens here. we store as less as possible
  // for a better memory usage and flexibility.
  // the generated tokens should be consumed/dropped immediately after each lex.
  // users can store error tokens as needed by themselves as needed.
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
