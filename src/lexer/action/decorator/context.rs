use crate::lexer::action::{ActionInput, ActionOutput};

/// This struct provides the [`ActionInput`] and [`ActionOutput`]
/// in action decorators when the action is accepted.
#[derive(Debug, Clone)]
pub struct AcceptedActionOutputContext<InputType, OutputType> {
  /// The [`ActionInput`]. Might be `&ActionInput` or `&mut ActionInput`,
  /// depends on the specific action decorator you are using.
  pub input: InputType,
  /// The [`ActionOutput`]. Might also be `&ActionOutput`,
  /// depends on the specific action decorator you are using.
  pub output: OutputType,
}

macro_rules! impl_ctx {
  ($input:ty, $output:ty) => {
    impl<'text, Kind, StateRef, OptionErrorType> AcceptedActionOutputContext<$input, $output> {
      /// The [`Range::end`](crate::lexer::token::Range) of the token that this action will emit.
      #[inline]
      pub fn end(&self) -> usize {
        self.input.start() + self.output.digested
      }

      /// The text content of the token that this action will emit.
      #[inline]
      pub fn content(&self) -> &'text str {
        // we don't cache this slice since it might not be used frequently
        &self.input.text()[self.input.start()..self.end()]
      }

      /// The rest of the input text after this action is accepted.
      #[inline]
      pub fn rest(&self) -> &'text str {
        // we don't cache this slice since it might not be used frequently
        &self.input.text()[self.end()..]
      }
    }
  };
}

// ActionInput won't be consumed.
// ActionOutput won't be modified directly in the context.
impl_ctx!(&mut ActionInput<'text, StateRef>, ActionOutput<Kind, OptionErrorType>);
impl_ctx!(&mut ActionInput<'text, StateRef>, &ActionOutput<Kind, OptionErrorType>);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::token::MockTokenKind;

  fn create_input() -> ActionInput<'static, ()> {
    ActionInput::new("123", 1, ()).unwrap()
  }
  fn create_output() -> ActionOutput<MockTokenKind<()>, Option<()>> {
    ActionOutput {
      binding: MockTokenKind::new(()).into(),
      digested: 1,
      error: None::<()>,
    }
  }

  #[test]
  fn accepted_action_decorator_context() {
    // ensure the methods are working

    // &mut input and output
    AcceptedActionOutputContext {
      input: &mut create_input(),
      output: create_output(),
    }
    .end();

    // &mut input and &output
    AcceptedActionOutputContext {
      input: &mut create_input(),
      output: &create_output(),
    }
    .end();

    // ensure the value is correct
    assert_eq!(
      AcceptedActionOutputContext {
        input: &mut create_input(),
        output: create_output(),
      }
      .end(),
      2
    );
    assert_eq!(
      AcceptedActionOutputContext {
        input: &mut create_input(),
        output: create_output(),
      }
      .content(),
      "2"
    );
    assert_eq!(
      AcceptedActionOutputContext {
        input: &mut create_input(),
        output: create_output(),
      }
      .rest(),
      "3"
    );
  }
}
