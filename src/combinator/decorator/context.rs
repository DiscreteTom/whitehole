use crate::action::{Input, Output};
use std::ops::Range;

/// This struct provides the [`Input`] and [`Output`]
/// in combinator decorators when the combinator is accepted.
///
/// You can't construct this struct directly.
#[derive(Debug)]
pub struct AcceptedContext<InputType, OutputType> {
  /// The [`Input`].
  pub(super) input: InputType,
  /// The [`Output`].
  ///
  /// If the decorator can't consume the output, this will be `&Output`.
  pub(super) output: OutputType,
}

macro_rules! impl_ctx {
  ($input:ty, $output:ty) => {
    impl<'text, 'state, 'heap, 'output, Value, State, Heap> AcceptedContext<$input, $output> {
      /// Get the [`Input`] of this execution.
      #[inline]
      pub fn input(&self) -> &$input {
        // return non-mutable reference to prevent mem::swap and override `Input::start` and `Input::rest`.
        &self.input
      }

      /// See [`Input::start`].
      #[inline]
      pub const fn start(&self) -> usize {
        self.input.start()
      }

      /// See [`Input::state`].
      #[inline]
      pub const fn state(&mut self) -> &mut State {
        self.input.state
      }

      /// See [`Input::heap`].
      #[inline]
      pub const fn heap(&mut self) -> &mut Heap {
        self.input.heap
      }

      /// See [`Output::digested`].
      #[inline]
      pub const fn digested(&self) -> usize {
        self.output.digested
      }

      /// Get the rest of the input text after accepting this combinator.
      #[inline]
      pub fn rest(&self) -> &'text str {
        debug_assert!(self.output.digested <= self.input.rest().len());
        unsafe { self.input.rest().get_unchecked(self.digested()..) }
      }

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

      /// The text content accepted by this combinator.
      #[inline]
      pub fn content(&self) -> &'text str {
        debug_assert!(self.digested() <= self.input.rest().len());
        unsafe { self.input.rest().get_unchecked(..self.digested()) }
      }

      /// Take the [`Output`].
      #[inline]
      pub fn take(self) -> $output {
        self.output
      }

      /// Split the instance into the [`Input`] and [`Output`].
      #[inline]
      pub fn split(self) -> ($input, $output) {
        (self.input, self.output)
      }
    }
  };
}

// Input will always be consumed.
// Output won't be modified directly in the context, but can be consumed.
impl_ctx!(
  Input<'text, &'state mut State, &'heap mut Heap>,
  Output<Value>
);
impl_ctx!(
  Input<'text, &'state mut State, &'heap mut Heap>,
  &'output Output<Value>
);

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! ctx {
    () => {
      ctx!((), ())
    };
    ($state:expr, $heap:expr) => {
      AcceptedContext {
        input: Input::new("123", 1, &mut $state, &mut $heap).unwrap(),
        output: Output {
          value: (),
          digested: 1,
        },
      }
    };
  }

  macro_rules! ctx_ref {
    () => {
      ctx_ref!((), ())
    };
    ($state:expr, $heap:expr) => {
      AcceptedContext {
        input: Input::new("123", 1, &mut $state, &mut $heap).unwrap(),
        output: &Output {
          value: (),
          digested: 1,
        },
      }
    };
  }

  #[test]
  fn accepted_decorator_context() {
    // getters
    assert_eq!(ctx!().input().rest(), "123");
    assert_eq!(ctx!().input().next(), '1');
    assert_eq!(ctx!().start(), 1);
    assert_eq!(ctx!().digested(), 1);
    assert_eq!(ctx!().rest(), "23");
    assert_eq!(ctx!().end(), 2);
    assert_eq!(ctx!().range(), 1..2);
    assert_eq!(ctx!().content(), "1");
    assert_eq!(ctx_ref!().input().rest(), "123");
    assert_eq!(ctx_ref!().input().next(), '1');
    assert_eq!(ctx_ref!().start(), 1);
    assert_eq!(ctx_ref!().digested(), 1);
    assert_eq!(ctx_ref!().rest(), "23");
    assert_eq!(ctx_ref!().end(), 2);
    assert_eq!(ctx_ref!().range(), 1..2);
    assert_eq!(ctx_ref!().content(), "1");

    // mutable state & heap
    let mut state = 0;
    *ctx!(state, ()).state() = 1;
    assert_eq!(*ctx!(state, ()).state(), 1);
    let mut heap = 0;
    *ctx!((), heap).heap() = 1;
    assert_eq!(*ctx!((), heap).heap(), 1);
    let mut state = 0;
    *ctx_ref!(state, ()).state() = 1;
    assert_eq!(*ctx_ref!(state, ()).state(), 1);
    let mut heap = 0;
    *ctx_ref!((), heap).heap() = 1;
    assert_eq!(*ctx_ref!((), heap).heap(), 1);

    // take & split
    assert_eq!(ctx!().take().digested, 1);
    assert_eq!(ctx!().take().map(|_| 1).value, 1);
    assert_eq!(ctx!().split().1.map(|_| 1).value, 1);
    assert_eq!(ctx!().split().0.reborrow().start(), 1);
    assert_eq!(ctx_ref!().take().digested, 1);
    assert_eq!(ctx_ref!().split().0.reborrow().start(), 1);
  }
}
