use crate::action::{Input, Output};
use std::ops::Range;

/// This struct provides the [`Input`] and [`Output`]
/// in combinator decorators when the combinator is accepted.
#[derive(Debug)]
pub struct AcceptedContext<InputType, OutputType> {
  /// The `&mut Input`.
  pub input: InputType,
  /// The [`Output`].
  ///
  /// If the decorator can't consume the output, this will be `&Output`.
  pub output: OutputType,
}

macro_rules! impl_ctx {
  ($input:ty, $output:ty) => {
    impl<'text, Value, StateRef, HeapRef> AcceptedContext<$input, $output> {
      /// How many bytes are digested by this combinator.
      #[inline]
      pub fn digested(&self) -> usize {
        self.input.rest().len() - self.output.rest.len()
      }

      /// The end index in bytes in the whole input text.
      ///
      /// Shortcut for `self.input.start() + self.digested()`.
      #[inline]
      pub fn end(&self) -> usize {
        self.input.start() + self.digested()
      }

      /// The byte range of the digested text in the whole input text.
      ///
      /// Shortcut for `self.input.start()..self.end()`.
      #[inline]
      pub fn range(&self) -> Range<usize> {
        self.input.start()..self.end()
      }

      /// The text content accepted by this combinator.
      #[inline]
      pub fn content(&self) -> &'text str {
        // we don't cache this slice since it might not be used frequently
        // SAFETY: for normal cases, the `output.rest` and `input.rest` are slices of the same string
        // and the `output.rest` is always a suffix of `input.rest` so it's safe to get the slice unchecked.
        // but in case the user gives a wrong output, we still use `debug_assert!` to check it.
        debug_assert!(self.digested() <= self.input.rest().len());
        unsafe { self.input.rest().get_unchecked(..self.digested()) }
      }
    }
  };
}

// Input won't be consumed and is always mutable.
// Output won't be modified directly in the context, but can be consumed.
impl_ctx!(&mut Input<'text, StateRef, HeapRef>, Output<'text, Value>);
impl_ctx!(&mut Input<'text, StateRef, HeapRef>, &Output<'text, Value>);

#[cfg(test)]
mod tests {
  use super::*;

  fn create_input() -> Input<'static, (), ()> {
    Input::new("123", 1, (), ()).unwrap()
  }
  fn create_output() -> Output<'static, ()> {
    Output {
      value: (),
      rest: "23",
    }
  }

  #[test]
  fn accepted_decorator_context() {
    // ensure the methods are working

    // &mut input and output
    AcceptedContext {
      input: &mut create_input(),
      output: create_output(),
    }
    .end();

    // &mut input and &output
    AcceptedContext {
      input: &mut create_input(),
      output: &create_output(),
    }
    .end();

    // ensure the value is correct
    assert_eq!(
      AcceptedContext {
        input: &mut create_input(),
        output: create_output(),
      }
      .digested(),
      1
    );
    assert_eq!(
      AcceptedContext {
        input: &mut create_input(),
        output: create_output(),
      }
      .end(),
      2
    );
    assert_eq!(
      AcceptedContext {
        input: &mut create_input(),
        output: create_output(),
      }
      .content(),
      "1"
    );
  }

  #[test]
  #[should_panic]
  fn wrong_output() {
    // ensure the panic when the output is wrong
    AcceptedContext {
      input: &mut create_input(),
      output: Output {
        value: (),
        rest: "4567",
      },
    }
    .content();
  }
}
