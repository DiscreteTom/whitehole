//! Digest-able byte sequence. See [`Digest`].

/// A digest-able byte sequence.
///
/// Built-in implementations are provided for `&[u8]` and `&str`.
pub trait Digest {
  /// Validate if it is ok to digest the first `n` bytes.
  ///
  /// For both `&[u8]` and `&str`, this method will
  /// require `n` is no greater than the length of `self`.
  /// For `&str`, this method will also require `n` is a valid UTF-8 boundary.
  fn validate(&self, n: usize) -> bool;

  /// Convert self to a byte slice.
  fn as_bytes(&self) -> &[u8];

  /// Digest the next `n` bytes. Return the rest.
  /// # Safety
  /// You should ensure that `n` is valid according to [`Digest::validate`].
  /// This will be checked using [`debug_assert!`].
  unsafe fn digest_unchecked(&self, n: usize) -> Self;

  /// Return the first `n` bytes.
  /// # Safety
  /// You should ensure that `n` is valid according to [`Digest::validate`].
  /// This will be checked using [`debug_assert!`].
  unsafe fn span_unchecked(&self, n: usize) -> Self;
}

impl Digest for &[u8] {
  #[inline]
  fn validate(&self, n: usize) -> bool {
    n <= self.len()
  }

  #[inline]
  fn as_bytes(&self) -> &[u8] {
    self
  }

  #[inline]
  unsafe fn digest_unchecked(&self, n: usize) -> Self {
    debug_assert!(self.validate(n));
    self.get_unchecked(n..)
  }

  #[inline]
  unsafe fn span_unchecked(&self, n: usize) -> Self {
    debug_assert!(self.validate(n));
    self.get_unchecked(..n)
  }
}

impl Digest for &str {
  #[inline]
  fn validate(&self, n: usize) -> bool {
    self.is_char_boundary(n)
  }

  #[inline]
  fn as_bytes(&self) -> &[u8] {
    <str>::as_bytes(self)
  }

  #[inline]
  unsafe fn digest_unchecked(&self, n: usize) -> Self {
    debug_assert!(self.validate(n));
    self.get_unchecked(n..)
  }

  #[inline]
  unsafe fn span_unchecked(&self, n: usize) -> Self {
    debug_assert!(self.validate(n));
    self.get_unchecked(..n)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn digest_bytes() {
    let bytes = b"123" as &[u8];
    assert!(bytes.validate(0));
    assert!(bytes.validate(1));
    assert!(bytes.validate(2));
    assert!(bytes.validate(3));
    assert!(!bytes.validate(4));
    assert_eq!(unsafe { bytes.digest_unchecked(0) }, b"123");
    assert_eq!(unsafe { bytes.digest_unchecked(1) }, b"23");
    assert_eq!(unsafe { bytes.digest_unchecked(2) }, b"3");
    assert_eq!(unsafe { bytes.digest_unchecked(3) }, b"");
    assert_eq!(unsafe { bytes.span_unchecked(0) }, b"");
    assert_eq!(unsafe { bytes.span_unchecked(1) }, b"1");
    assert_eq!(unsafe { bytes.span_unchecked(2) }, b"12");
    assert_eq!(unsafe { bytes.span_unchecked(3) }, b"123");
  }

  #[test]
  #[should_panic]
  fn digest_bytes_overflow() {
    let bytes = b"123" as &[u8];
    unsafe { bytes.digest_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn digest_bytes_span_overflow() {
    let bytes = b"123" as &[u8];
    unsafe { bytes.span_unchecked(4) };
  }

  #[test]
  fn digest_str() {
    let text = "123";
    assert!(text.validate(0));
    assert!(text.validate(1));
    assert!(text.validate(2));
    assert!(text.validate(3));
    assert!(!text.validate(4));
    assert_eq!(unsafe { text.digest_unchecked(0) }, "123");
    assert_eq!(unsafe { text.digest_unchecked(1) }, "23");
    assert_eq!(unsafe { text.digest_unchecked(2) }, "3");
    assert_eq!(unsafe { text.digest_unchecked(3) }, "");
    assert_eq!(unsafe { text.span_unchecked(0) }, "");
    assert_eq!(unsafe { text.span_unchecked(1) }, "1");
    assert_eq!(unsafe { text.span_unchecked(2) }, "12");
    assert_eq!(unsafe { text.span_unchecked(3) }, "123");
  }

  #[test]
  #[should_panic]
  fn digest_str_invalid_code_point() {
    let text = "好";
    unsafe { text.digest_unchecked(1) };
  }

  #[test]
  #[should_panic]
  fn digest_str_span_invalid_code_point() {
    let text = "好";
    unsafe { text.span_unchecked(1) };
  }

  #[test]
  #[should_panic]
  fn digest_str_overflow() {
    let text = "123";
    unsafe { text.digest_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn digest_str_span_overflow() {
    let text = "123";
    unsafe { text.span_unchecked(4) };
  }
}
