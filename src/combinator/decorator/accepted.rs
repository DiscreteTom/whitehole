use crate::{action::Output, digest::Digest, instant::Instant};
use std::{
  ops::{Range, RangeFrom, RangeTo},
  slice::SliceIndex,
};

/// This struct provides the [`Instant`] and [`Output`]
/// in combinator decorators when the combinator is accepted.
///
/// You can't construct or modify this struct directly.
/// This is to ensure the [`Instant`] and [`Output`] are consistent
/// so we can skip some runtime checks.
#[derive(Debug, Clone)]
pub struct Accepted<'instant, TextRef, Value> {
  instant: &'instant Instant<TextRef>,
  output: Output<Value>,
}

impl<'instant, TextRef, Value> Accepted<'instant, TextRef, Value> {
  /// Create a new instance.
  ///
  /// This is only used internally by the library.
  #[inline]
  pub(super) const fn new(instant: &'instant Instant<TextRef>, output: Output<Value>) -> Self {
    Accepted { instant, output }
  }

  /// Get the [`Instant`] of this execution.
  #[inline]
  pub const fn instant(&self) -> &'instant Instant<TextRef> {
    // don't make `Self::instant` public. this is to prevent `mem::swap` and override `Instant::rest`.
    self.instant
  }

  /// Get the [`Output`] of this execution.
  #[inline]
  pub const fn output(&self) -> &Output<Value> {
    // return non-mutable reference to prevent mem::swap and override `Output::digested`.
    &self.output
  }

  /// Consume the instance and take the [`Output`].
  #[inline]
  pub fn take(self) -> Output<Value> {
    self.output
  }

  /// How many bytes will be digested by this accepted execution.
  ///
  /// See [`Output::digested`].
  #[inline]
  pub const fn digested(&self) -> usize {
    self.output.digested
  }

  /// The start index of the accepted content in the whole input text, in bytes.
  #[inline]
  pub const fn start(&self) -> usize {
    self.instant.digested()
  }

  /// The end index of the accepted content in the whole input text, in bytes.
  #[inline]
  pub const fn end(&self) -> usize {
    debug_assert!(usize::MAX - self.start() >= self.digested());
    unsafe { self.start().unchecked_add(self.digested()) }
  }

  /// The byte range of the digested content in the whole input text.
  ///
  /// Shortcut for `self.start()..self.end()`.
  #[inline]
  pub const fn range(&self) -> Range<usize> {
    self.start()..self.end()
  }
}

impl<'text, Text: ?Sized + Digest, Value> Accepted<'_, &'text Text, Value> {
  /// The text content accepted by this execution.
  #[inline]
  pub fn content(&self) -> &'text Text
  where
    RangeTo<usize>: SliceIndex<Text, Output = Text>,
  {
    debug_assert!(self.instant.rest().validate(self.output.digested));
    unsafe { self.instant.rest().get_unchecked(..self.digested()) }
  }

  /// Get the rest of the input text after accepting this combinator.
  #[inline]
  pub fn after(&self) -> &'text Text
  where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    debug_assert!(self.instant.rest().validate(self.output.digested));
    unsafe { self.instant.rest().get_unchecked(self.digested()..) }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::instant::Instant;

  macro_rules! ctx {
    () => {
      Accepted::new(
        &unsafe { Instant::new("0123").to_digested_unchecked(1) },
        Output {
          value: (),
          digested: 1,
        },
      )
    };
  }

  #[test]
  fn make_sure_accepted_clone_able() {
    let _ = ctx!().clone();
  }

  #[test]
  fn test_accepted() {
    // getters
    assert_eq!(ctx!().instant().rest(), "123");
    assert_eq!(ctx!().output().digested, 1);
    assert_eq!(ctx!().digested(), 1);
    assert_eq!(ctx!().start(), 1);
    assert_eq!(ctx!().end(), 2);
    assert_eq!(ctx!().range(), 1..2);
    assert_eq!(ctx!().content(), "1");
    assert_eq!(ctx!().after(), "23");

    // take
    assert_eq!(ctx!().take().digested, 1);
    assert_eq!(ctx!().take().map(|_| 1).value, 1);
  }
}
