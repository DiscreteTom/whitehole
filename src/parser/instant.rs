/// The instantaneous state of a parser (a.k.a the "configuration" in the automata theory).
///
/// This is cheap to clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instant<'text> {
  /// See [`Self::text`].
  text: &'text str,
  /// See [`Self::rest`].
  rest: &'text str,
  /// See [`Self::digested`].
  digested: usize,
}

impl<'text> Instant<'text> {
  /// Create a new instance with the given text.
  /// [`Self::digested`] will be set to `0`.
  #[inline]
  pub const fn new(text: &'text str) -> Self {
    Instant {
      text,
      rest: text,
      digested: 0,
    }
  }

  /// The whole input text.
  ///
  /// This is cheap to call because the value is stored in this struct.
  /// This will never be mutated after the creation of this instance.
  #[inline]
  pub const fn text(&self) -> &'text str {
    self.text
  }
  /// How many bytes are already digested.
  ///
  /// This is cheap to call because the value is stored in this struct.
  #[inline]
  pub const fn digested(&self) -> usize {
    self.digested
  }
  /// The undigested text.
  ///
  /// This is cheap to call because the value is stored in this struct.
  #[inline]
  pub const fn rest(&self) -> &'text str {
    self.rest
  }

  /// Digest the next `n` bytes.
  /// [`Self::rest`] will be updated automatically.
  /// # Safety
  /// You should ensure that `n` is a valid UTF-8 boundary.
  /// This will be checked using [`debug_assert!`].
  #[inline]
  pub unsafe fn digest_unchecked(&mut self, digested: usize) {
    debug_assert!(self.rest.is_char_boundary(digested));
    self.digested = digested;
    self.rest = self.text.get_unchecked(digested..);
  }

  // TODO: add digest
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn instant_new() {
    let state = Instant::new("123");
    assert_eq!(state.digested(), 0);
    assert_eq!(state.rest(), "123");
    assert_eq!(state.text(), "123");
  }

  #[test]
  fn instant_digest_unchecked() {
    let mut state = Instant::new("123");
    unsafe { state.digest_unchecked(2) };
    assert_eq!(state.digested(), 2);
    assert_eq!(state.rest(), "3");
    assert_eq!(state.text(), "123");
    // TODO: more tests
  }
}
