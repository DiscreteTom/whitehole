#[derive(Clone)]
pub struct LexerState<'text> {
  /// See [`Self::text`].
  text: &'text str,
  /// See [`Self::digested`].
  digested: usize,
  // we don't store error tokens here. we store as less as possible
  // for a better memory usage and flexibility.
  // the generated tokens should be consumed/dropped immediately after each lex.
  // users can store error tokens as needed by themselves as needed.
}

impl<'text> LexerState<'text> {
  /// Create a new lexer state with the given text.
  /// [`Self::digested`] will be 0.
  pub fn new(text: &'text str) -> Self {
    LexerState { text, digested: 0 }
  }

  /// The whole input text.
  pub fn text(&self) -> &'text str {
    self.text
  }
  /// How many bytes are digested.
  pub fn digested(&self) -> usize {
    self.digested
  }

  /// Get the rest of the text which is not digested.
  pub fn rest(&self) -> &'text str {
    &self.text[self.digested..]
  }

  /// Digest the given number of bytes.
  pub fn digest(&mut self, n: usize) {
    self.digested += n;
  }
}
