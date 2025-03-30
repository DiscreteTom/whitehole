use crate::{action::Output, digest::Digest, instant::Instant};
use std::{
  ops::{Range, RangeFrom, RangeTo},
  slice::SliceIndex,
};

/// This struct provides the [`Instant`], `&mut State`, `&mut Heap` and [`Output`]
/// in combinator decorators when the original combinator is accepted.
#[derive(Debug)]
pub struct Accepted<InstantRef, StateRef, HeapRef, Value> {
  /// The `&mut State`.
  /// See [`Parser::state`](crate::parser::Parser::state).
  pub state: StateRef,

  /// The `&mut Heap`.
  /// See [`Parser::heap`](crate::parser::Parser::heap).
  pub heap: HeapRef,

  /// See [`Self::instant`].
  instant: InstantRef,
  /// See [`Self::output`].
  output: Output<Value>,
}

impl<'instant, 'text, Text: ?Sized + Digest, StateRef, HeapRef, Value>
  Accepted<&'instant Instant<&'text Text>, StateRef, HeapRef, Value>
{
  /// Create a new instance.
  /// # Safety
  /// The caller must ensure the [`Output::digested`]
  /// is valid according to [`Digest::validate`] against [`Instant::rest`].
  /// This will be checked using [`debug_assert`].
  #[inline]
  pub unsafe fn new_unchecked(
    instant: &'instant Instant<&'text Text>,
    output: Output<Value>,
    state: StateRef,
    heap: HeapRef,
  ) -> Self {
    debug_assert!(instant.rest().validate(output.digested));
    Accepted {
      instant,
      output,
      state,
      heap,
    }
  }
}

impl<'instant, TextRef, StateRef, HeapRef, Value>
  Accepted<&'instant Instant<TextRef>, StateRef, HeapRef, Value>
{
  /// Get the [`Parser::instant`](crate::parser::Parser::instant) of this execution.
  ///
  /// You can't modify this directly.
  /// This is to ensure [`Self::instant`] and [`Self::output`] are consistent
  /// so we can skip some runtime checks.
  #[inline]
  pub const fn instant(&self) -> &'instant Instant<TextRef> {
    self.instant
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

impl<InstantRef, StateRef, HeapRef, Value> Accepted<InstantRef, StateRef, HeapRef, Value> {
  /// Get the [`Output`] of this execution.
  ///
  /// You can't modify this directly.
  /// This is to ensure [`Self::instant`] and [`Self::output`] are consistent
  /// so we can skip some runtime checks.
  #[inline]
  pub const fn output(&self) -> &Output<Value> {
    &self.output
  }

  /// How many bytes will be digested by this accepted execution.
  ///
  /// See [`Output::digested`].
  #[inline]
  pub const fn digested(&self) -> usize {
    self.output.digested
  }

  /// Consume the instance and take the [`Output`].
  #[inline]
  pub fn take(self) -> Output<Value> {
    self.output
  }
}

impl<'text, Text: ?Sized + Digest, StateRef, HeapRef, Value>
  Accepted<&Instant<&'text Text>, StateRef, HeapRef, Value>
{
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
      unsafe {
        Accepted::new_unchecked(
          &Instant::new("0123").to_digested_unchecked(1),
          Output {
            value: (),
            digested: 1,
          },
          &mut (),
          &mut (),
        )
      }
    };
  }

  macro_rules! ctx_bytes {
    () => {
      unsafe {
        Accepted::new_unchecked(
          &Instant::new(b"0123" as &[u8]).to_digested_unchecked(1),
          Output {
            value: (),
            digested: 1,
          },
          &mut (),
          &mut (),
        )
      }
    };
  }

  #[test]
  #[should_panic]
  fn accepted_new_unchecked_invalid_digested() {
    unsafe {
      Accepted::new_unchecked(
        &Instant::new("0123").to_digested_unchecked(1),
        Output {
          value: (),
          digested: 4,
        },
        &mut (),
        &mut (),
      )
    };
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

    // debug
    let _ = format!("{:?}", ctx!());
  }

  #[test]
  fn test_accepted_bytes() {
    // getters
    assert_eq!(ctx_bytes!().instant().rest(), b"123");
    assert_eq!(ctx_bytes!().output().digested, 1);
    assert_eq!(ctx_bytes!().digested(), 1);
    assert_eq!(ctx_bytes!().start(), 1);
    assert_eq!(ctx_bytes!().end(), 2);
    assert_eq!(ctx_bytes!().range(), 1..2);
    assert_eq!(ctx_bytes!().content(), b"1");
    assert_eq!(ctx_bytes!().after(), b"23");

    // take
    assert_eq!(ctx_bytes!().take().digested, 1);
    assert_eq!(ctx_bytes!().take().map(|_| 1).value, 1);

    // debug
    let _ = format!("{:?}", ctx_bytes!());
  }
}
