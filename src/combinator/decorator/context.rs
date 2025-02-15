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
#[derive(Debug)]
pub struct AcceptedContext<'a, TextRef, Value> {
  instant: &'a Instant<TextRef>,
  output: Output<Value>,
}

impl<'a, TextRef, Value> AcceptedContext<'a, TextRef, Value> {
  /// Create a new instance.
  ///
  /// This is only used internally by the library.
  #[inline]
  pub(super) const fn new(instant: &'a Instant<TextRef>, output: Output<Value>) -> Self {
    AcceptedContext { instant, output }
  }

  /// Get the [`Output`] of this execution.
  #[inline]
  pub const fn output(&self) -> &Output<Value> {
    // return non-mutable reference to prevent mem::swap and override `Output::digested`.
    &self.output
  }

  /// Take the [`Output`].
  ///
  /// To get the [`Input`] as well, use [`Self::split`].
  #[inline]
  pub fn take(self) -> Output<Value> {
    self.output
  }

  /// Split the instance into the [`Input`] and [`Output`].
  ///
  /// To get the [`Output`] only, use [`Self::take`].
  #[inline]
  pub fn split(self) -> (&'a Instant<TextRef>, Output<Value>) {
    (self.instant, self.output)
  }
}

impl<'a, TextRef, Value> AcceptedContext<'a, TextRef, Value> {
  #[inline]
  pub const fn instant(&self) -> &'a Instant<TextRef> {
    self.instant
  }

  #[inline]
  pub const fn start(&self) -> usize {
    self.instant.digested()
  }
}

impl<TextRef, Value> AcceptedContext<'_, TextRef, Value> {
  /// See [`Output::digested`].
  #[inline]
  pub const fn digested(&self) -> usize {
    self.output.digested
  }
}

impl<TextRef, Value> AcceptedContext<'_, TextRef, Value> {
  /// The end index in bytes in the whole input text.
  #[inline]
  pub fn end(&self) -> usize {
    debug_assert!(usize::MAX - self.start() >= self.digested());
    unsafe { self.start().unchecked_add(self.digested()) }
  }

  /// The byte range of the digested text in the whole input text.
  ///
  /// Shortcut for `self.start()..self.end()`.
  #[inline]
  pub fn range(&self) -> Range<usize> {
    self.start()..self.end()
  }
}

impl<'a, Text: ?Sized + Digest, Value> AcceptedContext<'_, &'a Text, Value> {
  /// Get the rest of the input text after accepting this combinator.
  #[inline]
  pub fn rest(&self) -> &'a Text
  where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    debug_assert!(self.instant.rest().validate(self.output.digested));
    unsafe { self.instant.rest().get_unchecked(self.digested()..) }
  }

  /// The text content accepted by this combinator.
  #[inline]
  pub fn content(&self) -> &'a Text
  where
    RangeTo<usize>: SliceIndex<Text, Output = Text>,
  {
    debug_assert!(self.instant.rest().validate(self.output.digested));
    unsafe { self.instant.rest().get_unchecked(..self.digested()) }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::instant::Instant;

  macro_rules! ctx {
    () => {
      ctx!((), ())
    };
    ($state:expr, $heap:expr) => {
      AcceptedContext::new(
        &unsafe { Instant::new("0123").to_digested_unchecked(1) },
        Output {
          value: (),
          digested: 1,
        },
      )
    };
  }

  #[test]
  fn accepted_decorator_context() {
    // getters
    assert_eq!(ctx!().instant().rest(), "123");
    assert_eq!(ctx!().output().digested, 1);
    assert_eq!(ctx!().start(), 1);
    assert_eq!(ctx!().digested(), 1);
    assert_eq!(ctx!().rest(), "23");
    assert_eq!(ctx!().end(), 2);
    assert_eq!(ctx!().range(), 1..2);
    assert_eq!(ctx!().content(), "1");

    // take & split
    assert_eq!(ctx!().take().digested, 1);
    assert_eq!(ctx!().take().map(|_| 1).value, 1);
    assert_eq!(ctx!().split().1.map(|_| 1).value, 1);
    assert_eq!(ctx!().split().0.digested(), 1);
  }
}
