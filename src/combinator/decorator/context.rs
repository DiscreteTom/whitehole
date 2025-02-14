use crate::{
  action::{Input, Output},
  digest::Digest,
};
use std::ops::Range;

/// This struct provides the [`Input`] and [`Output`]
/// in combinator decorators when the combinator is accepted.
///
/// You can't construct or modify this struct directly.
/// This is to ensure the [`Input`] and [`Output`] are consistent
/// so we can skip some runtime checks.
#[derive(Debug)]
pub struct AcceptedContext<InputType, OutputType> {
  /// The [`Input`].
  input: InputType,
  /// The [`Output`].
  output: OutputType,
}

impl<InputType, OutputType> AcceptedContext<InputType, OutputType> {
  /// Create a new instance.
  ///
  /// This is only used internally by the library.
  #[inline]
  pub(super) const fn new(input: InputType, output: OutputType) -> Self {
    AcceptedContext { input, output }
  }

  /// Get the [`Input`] of this execution.
  #[inline]
  pub const fn input(&self) -> &InputType {
    // return non-mutable reference to prevent mem::swap and override `Input::instant`.
    &self.input
  }

  /// Get the [`Output`] of this execution.
  #[inline]
  pub const fn output(&self) -> &OutputType {
    // return non-mutable reference to prevent mem::swap and override `Output::digested`.
    &self.output
  }

  /// Take the [`Output`].
  ///
  /// To get the [`Input`] as well, use [`Self::split`].
  #[inline]
  pub fn take(self) -> OutputType {
    self.output
  }

  /// Split the instance into the [`Input`] and [`Output`].
  ///
  /// To get the [`Output`] only, use [`Self::take`].
  #[inline]
  pub fn split(self) -> (InputType, OutputType) {
    (self.input, self.output)
  }
}

impl<TextRef, State, Heap, OutputType>
  AcceptedContext<Input<TextRef, &mut State, &mut Heap>, OutputType>
{
  /// The `self.input().instant().digested()`.
  #[inline]
  pub const fn start(&self) -> usize {
    self.input.instant().digested()
  }

  /// See [`Input::state`].
  #[inline]
  pub const fn state(&mut self) -> &mut State {
    // since `Self::input` returns non-mutable reference, we have to provide this to get mutable reference.
    self.input.state
  }

  /// See [`Input::heap`].
  #[inline]
  pub const fn heap(&mut self) -> &mut Heap {
    // since `Self::input` returns non-mutable reference, we have to provide this to get mutable reference.
    self.input.heap
  }
}

impl<InputType, Value> AcceptedContext<InputType, Output<Value>> {
  /// See [`Output::digested`].
  #[inline]
  pub const fn digested(&self) -> usize {
    self.output.digested
  }
}

impl<TextRef, Value, State, Heap>
  AcceptedContext<Input<TextRef, &mut State, &mut Heap>, Output<Value>>
{
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

impl<'a, Text: ?Sized, Value, State, Heap>
  AcceptedContext<Input<&'a Text, &mut State, &mut Heap>, Output<Value>>
where
  &'a Text: Digest,
{
  /// Get the rest of the input text after accepting this combinator.
  #[inline]
  pub fn rest(&self) -> &'a Text {
    debug_assert!(self.input.validate(self.output.digested));
    unsafe {
      self
        .input
        .instant()
        .rest()
        .digest_unchecked(self.digested())
    }
  }

  /// The text content accepted by this combinator.
  #[inline]
  pub fn content(&self) -> &'a Text {
    debug_assert!(self.input.validate(self.output.digested));
    unsafe { self.input.instant().rest().span_unchecked(self.digested()) }
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
        Input::new(
          {
            let mut instant = Instant::new("0123");
            unsafe { instant.digest_unchecked(1) };
            instant
          },
          &mut $state,
          &mut $heap,
        ),
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
    assert_eq!(ctx!().input().instant().rest(), "123");
    assert_eq!(ctx!().output().digested, 1);
    assert_eq!(ctx!().start(), 1);
    assert_eq!(ctx!().digested(), 1);
    assert_eq!(ctx!().rest(), "23");
    assert_eq!(ctx!().end(), 2);
    assert_eq!(ctx!().range(), 1..2);
    assert_eq!(ctx!().content(), "1");

    // mutable state & heap
    let mut state = 0;
    *ctx!(state, ()).state() = 1;
    assert_eq!(*ctx!(state, ()).state(), 1);
    let mut heap = 0;
    *ctx!((), heap).heap() = 1;
    assert_eq!(*ctx!((), heap).heap(), 1);

    // take & split
    assert_eq!(ctx!().take().digested, 1);
    assert_eq!(ctx!().take().map(|_| 1).value, 1);
    assert_eq!(ctx!().split().1.map(|_| 1).value, 1);
    assert_eq!(ctx!().split().0.reborrow().instant().digested(), 1);
  }
}
