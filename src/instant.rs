//! The instantaneous state of a parser (a.k.a the "configuration" in the automata theory).
//! See [`Instant`].

use crate::digest::Digest;
use std::{ops::RangeFrom, slice::SliceIndex};

/// The instantaneous state of a parser (a.k.a the "configuration" in the automata theory).
///
/// This is cheap to clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instant<TextRef> {
  /// See [`Self::text`].
  text: TextRef,
  /// See [`Self::rest`].
  rest: TextRef,
  /// See [`Self::digested`].
  digested: usize,
}

impl<'a, Text: ?Sized> Instant<&'a Text> {
  /// Create a new instance with the given text.
  /// [`Self::digested`] will be set to `0`.
  #[inline]
  pub const fn new(text: &'a Text) -> Self {
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
  pub const fn text(&self) -> &'a Text {
    self.text
  }

  /// The undigested text. This might be an empty string.
  ///
  /// This is cheap to call because the value is stored in this struct.
  #[inline]
  pub const fn rest(&self) -> &'a Text {
    self.rest
  }

  /// Digest the next `n` bytes.
  /// This will update [`Self::rest`] and [`Self::digested`].
  /// # Safety
  /// You should ensure that `n` is valid according to [`Digest::validate`].
  /// This will be checked using [`debug_assert!`].
  #[inline]
  pub unsafe fn digest_unchecked(&mut self, n: usize)
  where
    Text: Digest,
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    // TODO: rename to shift_unchecked
    debug_assert!(self.rest.validate(n));
    self.rest = self.rest.get_unchecked(n..);
    self.digested = self.digested.unchecked_add(n);
  }

  /// Construct a new instance by digesting `n` bytes from [`Self::rest`].
  ///
  /// This is cheap to call.
  /// # Safety
  /// You should ensure that `n` is valid according to [`Digest::validate`].
  /// This will be checked using [`debug_assert!`].
  #[inline]
  pub unsafe fn shift_unchecked(&self, n: usize) -> Self
  where
    Text: Digest,
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    // TODO: rename to digest_unchecked
    let mut instant = self.clone();
    instant.digest_unchecked(n);
    instant
  }
}

impl<TextRef> Instant<TextRef> {
  /// How many bytes are already digested.
  ///
  /// This is cheap to call because the value is stored in this struct.
  #[inline]
  pub const fn digested(&self) -> usize {
    self.digested
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn instant_new_getters() {
    let i = Instant::new("123");
    assert_eq!(i.digested(), 0);
    assert_eq!(i.rest(), "123");
    assert_eq!(i.text(), "123");
  }

  #[test]
  fn instant_clone_eq() {
    let i = Instant::new("123");
    let j = i.clone();
    assert_eq!(i, j);
  }

  #[test]
  fn instant_debug() {
    let i = Instant::new("123");
    assert_eq!(
      format!("{:?}", i),
      "Instant { text: \"123\", rest: \"123\", digested: 0 }"
    );
  }

  #[test]
  fn instant_bytes_digest_unchecked() {
    let mut i = Instant::new(b"123" as &[u8]);
    unsafe { i.digest_unchecked(1) };
    assert_eq!(i.digested(), 1);
    assert_eq!(i.rest(), b"23");
    assert_eq!(i.text(), b"123");
    unsafe { i.digest_unchecked(1) };
    assert_eq!(i.digested(), 2);
    assert_eq!(i.rest(), b"3");
    assert_eq!(i.text(), b"123");
  }

  #[test]
  #[should_panic]
  fn instant_bytes_digest_unchecked_overflow() {
    let mut i = Instant::new(b"123" as &[u8]);
    unsafe { i.digest_unchecked(4) };
  }

  #[test]
  fn instant_str_digest_unchecked() {
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
  fn instant_str_digest_unchecked_overflow() {
    let mut i = Instant::new("123");
    unsafe { i.digest_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn instant_str_digest_unchecked_invalid_code_point() {
    let mut i = Instant::new("好");
    unsafe { i.digest_unchecked(1) };
  }

  #[test]
  fn instant_shift_unchecked() {
    let instant = unsafe { Instant::new("123").shift_unchecked(1) };
    assert_eq!(instant.digested(), 1);
    assert_eq!(instant.rest(), "23");

    let instant = unsafe { Instant::new(b"123" as &[u8]).shift_unchecked(1) };
    assert_eq!(instant.digested(), 1);
    assert_eq!(instant.rest(), b"23");
  }

  #[test]
  #[should_panic]
  fn instant_shift_unchecked_invalid_utf8() {
    let _ = unsafe { Instant::new("好").shift_unchecked(1) };
  }

  #[test]
  #[should_panic]
  fn instant_bytes_shift_overflow() {
    let _ = unsafe { Instant::new(b"123" as &[u8]).shift_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn instant_str_shift_overflow() {
    let _ = unsafe { Instant::new("123").shift_unchecked(4) };
  }
}
