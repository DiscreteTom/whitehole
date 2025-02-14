//! Digest-able byte sequence. See [`Digest`].

use std::slice::SliceIndex;

/// A digest-able byte sequence.
///
/// Built-in implementations are provided for `[u8]` and [`str`].
pub trait Digest {
  /// Validate if it is ok to digest the first `n` bytes.
  ///
  /// For both `[u8]` and [`str`], this method will
  /// require `n` is no greater than the length of `self`.
  /// For [`str`], this method will also require `n` is a valid UTF-8 boundary.
  fn validate(&self, n: usize) -> bool;

  /// Convert self to a byte slice.
  fn as_bytes(&self) -> &[u8];

  /// Get a subslice of `self` if it is valid.
  fn get<I: SliceIndex<Self>>(&self, i: I) -> Option<&I::Output>;

  /// Get an unchecked subslice of `self` without bound checking.
  /// # Safety
  /// You should ensure the provided index is valid according to [`Digest::validate`].
  /// For a safe version, use [`Digest::get`].
  unsafe fn get_unchecked<I: SliceIndex<Self>>(&self, i: I) -> &I::Output;
}

impl Digest for [u8] {
  #[inline]
  fn validate(&self, n: usize) -> bool {
    n <= self.len()
  }

  #[inline]
  fn as_bytes(&self) -> &[u8] {
    self
  }

  #[inline]
  fn get<I: SliceIndex<Self>>(&self, i: I) -> Option<&I::Output> {
    self.get(i)
  }

  #[inline]
  unsafe fn get_unchecked<I: SliceIndex<Self>>(&self, i: I) -> &I::Output {
    self.get_unchecked(i)
  }
}

impl Digest for str {
  #[inline]
  fn validate(&self, n: usize) -> bool {
    self.is_char_boundary(n)
  }

  #[inline]
  fn as_bytes(&self) -> &[u8] {
    self.as_bytes()
  }

  #[inline]
  fn get<I: SliceIndex<Self>>(&self, i: I) -> Option<&I::Output> {
    self.get(i)
  }

  #[inline]
  unsafe fn get_unchecked<I: SliceIndex<Self>>(&self, i: I) -> &I::Output {
    self.get_unchecked(i)
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
    assert_eq!(bytes.as_bytes(), b"123");
    assert_eq!(<[u8] as Digest>::get(bytes, 0), Some(&b'1'));
    assert_eq!(<[u8] as Digest>::get(bytes, 0..), Some(b"123" as &[u8]));
    assert_eq!(unsafe { <[u8] as Digest>::get_unchecked(bytes, 0) }, &b'1');
    assert_eq!(
      unsafe { <[u8] as Digest>::get_unchecked(bytes, 0..) },
      b"123"
    );
  }

  #[test]
  fn digest_str() {
    let text = "好";
    assert!(text.validate(0));
    assert!(!text.validate(1));
    assert!(!text.validate(2));
    assert!(text.validate(3));
    assert!(!text.validate(4));
    assert_eq!(<str as Digest>::as_bytes(text), [229, 165, 189]);
    assert_eq!(<str as Digest>::get(text, 0..), Some("好"));
    assert_eq!(unsafe { <str as Digest>::get_unchecked(text, 0..) }, "好");
  }
}
