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
  /// The undigested text. This might be an empty string.
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
  pub unsafe fn digest_unchecked(&mut self, n: usize) {
    debug_assert!(self.rest.is_char_boundary(n));
    self.rest = self.rest.get_unchecked(n..);
    self.digested = self.digested.unchecked_add(n);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn instant_new() {
    let i = Instant::new("123");
    assert_eq!(i.digested(), 0);
    assert_eq!(i.rest(), "123");
    assert_eq!(i.text(), "123");
  }

  #[test]
  fn instant_digest_unchecked() {
    let mut i = Instant::new("123");
    unsafe { i.digest_unchecked(1) };
    assert_eq!(i.digested(), 1);
    assert_eq!(i.rest(), "23");
    assert_eq!(i.text(), "123");
    unsafe { i.digest_unchecked(1) };
    assert_eq!(i.digested(), 2);
    assert_eq!(i.rest(), "3");
    assert_eq!(i.text(), "123");
  }

  #[test]
  #[should_panic]
  fn instant_digest_unchecked_overflow() {
    let mut i = Instant::new("123");
    unsafe { i.digest_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn instant_digest_unchecked_invalid_code_point() {
    let mut i = Instant::new("好");
    unsafe { i.digest_unchecked(1) };
  }
}
