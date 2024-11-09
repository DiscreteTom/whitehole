use crate::combinator::{Input, Output};
use std::ops::Range;

/// This struct provides the [`Input`] and [`Output`]
/// in combinator decorators when the combinator is accepted.
#[derive(Debug, Clone)]
pub struct AcceptedOutputContext<InputType, OutputType> {
  /// The `&mut Input`.
  pub input: InputType,
  /// The [`Output`].
  ///
  /// Might also be `&Output`, depends on the specific decorator you are using.
  pub output: OutputType,
}

macro_rules! impl_ctx {
  ($input:ty, $output:ty) => {
    impl<'text, Kind, StateRef, HeapRef> AcceptedOutputContext<$input, $output> {
      /// Shorthand for `self.input.start() + self.output.digested`.
      pub fn end(&self) -> usize {
        self.input.start() + self.output.digested
      }

      /// Shorthand for `self.input.start()..self.end()`.
      pub fn range(&self) -> Range<usize> {
        self.input.start()..self.end()
      }

      /// The text content accepted by this combinator.
      pub fn content(&self) -> &'text str {
        // we don't cache this slice since it might not be used frequently
        &self.input.text()[self.range()]
      }

      /// The rest of the input text after this combinator is accepted.
      pub fn rest(&self) -> &'text str {
        // we don't cache this slice since it might not be used frequently
        &self.input.text()[self.end()..]
      }
    }
  };
}

// Input won't be consumed and is always mutable.
// Output won't be modified directly in the context, but can be consumed.
impl_ctx!(&mut Input<'text, StateRef, HeapRef>, Output<Kind>);
impl_ctx!(&mut Input<'text, StateRef, HeapRef>, &Output<Kind>);

#[cfg(test)]
mod tests {
  use super::*;

  fn create_input() -> Input<'static, (), ()> {
    Input::new("123", 1, (), ()).unwrap()
  }
  fn create_output() -> Output<()> {
    Output {
      kind: (),
      digested: 1,
    }
  }

  #[test]
  fn accepted_decorator_context() {
    // ensure the methods are working

    // &mut input and output
    AcceptedOutputContext {
      input: &mut create_input(),
      output: create_output(),
    }
    .end();

    // &mut input and &output
    AcceptedOutputContext {
      input: &mut create_input(),
      output: &create_output(),
    }
    .end();

    // ensure the value is correct
    assert_eq!(
      AcceptedOutputContext {
        input: &mut create_input(),
        output: create_output(),
      }
      .end(),
      2
    );
    assert_eq!(
      AcceptedOutputContext {
        input: &mut create_input(),
        output: create_output(),
      }
      .content(),
      "2"
    );
    assert_eq!(
      AcceptedOutputContext {
        input: &mut create_input(),
        output: create_output(),
      }
      .rest(),
      "3"
    );
  }
}
