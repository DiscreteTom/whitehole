#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexerState<'text> {
  /// See [`Self::text`].
  text: &'text str,
  /// See [`Self::digested`].
  digested: usize,
  /// See [`Self::trimmed`].
  trimmed: bool,
  // we don't store tokens here. we store as less as possible
  // for a better memory usage and flexibility.
  // the generated tokens should be consumed/dropped immediately after each lex.
  // users can store tokens as needed by themselves.
}

impl<'text> LexerState<'text> {
  /// Create a new lexer state with the given text.
  /// [`Self::digested`] will be set to `0`.
  #[inline]
  pub const fn new(text: &'text str) -> Self {
    LexerState {
      text,
      digested: 0,
      trimmed: text.len() == 0,
    }
  }

  /// The whole input text.
  #[inline]
  pub const fn text(&self) -> &'text str {
    self.text
  }
  /// How many bytes are digested.
  #[inline]
  pub const fn digested(&self) -> usize {
    self.digested
  }
  /// Whether the text is trimmed.
  #[inline]
  pub const fn trimmed(&self) -> bool {
    self.trimmed
  }

  /// Get the undigested text.
  #[inline]
  pub fn rest(&self) -> &'text str {
    &self.text[self.digested..]
  }

  /// Digest `n` bytes.
  /// The caller should ensure `n` is smaller than the rest text length.
  #[inline]
  pub fn digest(&mut self, n: usize) {
    debug_assert!(
      self.digested + n <= self.text.len(),
      "digest overflow, digested = {}, n = {}, text.len() = {}",
      self.digested,
      n,
      self.text.len()
    );

    self.digested += n;
    self.trimmed = self.digested == self.text.len();
  }

  /// Digest `n` bytes and set [`Self::trimmed`] to `true`.
  /// The caller should ensure `n` is smaller than the rest text length.
  #[inline]
  pub fn trim(&mut self, n: usize) {
    debug_assert!(
      self.digested + n <= self.text.len(),
      "digest overflow, digested = {}, n = {}, text.len() = {}",
      self.digested,
      n,
      self.text.len()
    );

    self.digested += n;
    self.trimmed = true;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_digest() {
    let mut state = LexerState::new("123");
    assert_eq!(state.digested(), 0);
    assert_eq!(state.trimmed(), false);

    state.digest(1);
    assert_eq!(state.digested(), 1);
    assert_eq!(state.trimmed(), false);

    state.digest(2);
    assert_eq!(state.digested(), 3);
    assert_eq!(state.trimmed(), true);
  }

  #[test]
  #[should_panic]
  fn test_digest_overflow() {
    let mut state = LexerState::new("123");
    state.digest(4);
  }

  #[test]
  fn test_trim() {
    let mut state = LexerState::new("123");
    assert_eq!(state.digested(), 0);
    assert_eq!(state.trimmed(), false);

    state.trim(1);
    assert_eq!(state.digested(), 1);
    assert_eq!(state.trimmed(), true);
  }

  #[test]
  #[should_panic]
  fn test_trim_overflow() {
    let mut state = LexerState::new("123");
    state.trim(4);
  }
}
