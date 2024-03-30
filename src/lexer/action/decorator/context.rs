use crate::lexer::action::{ActionInput, ActionOutput};

/// This struct provides the [`ActionInput`] and [`ActionOutput`]
/// in action decorators when the action is accepted.
pub struct AcceptedActionOutputContext<InputType, OutputType> {
  /// The [`ActionInput`]. Might also be `&ActionInput` or `&mut ActionInput`,
  /// depends on the specific action decorator you are using.
  pub input: InputType,
  /// The [`ActionOutput`]. Might also be `&ActionOutput` or `&mut ActionOutput`,
  /// depends on the specific action decorator you are using.
  pub output: OutputType,
}

// TODO: simplify these? or make _text/_start/_digested private?

/// This trait is used to unify [`ActionInput`],
/// `&ActionInput` and `&mut ActionInput` in
/// [`AcceptedActionDecoratorContext`]
pub trait AcceptedActionDecoratorContextInput<'text> {
  /// Don't use this. This is only used internally.
  fn _text(&self) -> &'text str;
  /// Don't use this. This is only used internally.
  fn _start(&self) -> usize;
}
macro_rules! impl_ctx_input {
  ($t:ty) => {
    impl<'text, ActionState> AcceptedActionDecoratorContextInput<'text> for $t {
      fn _text(&self) -> &'text str {
        self.text()
      }
      fn _start(&self) -> usize {
        self.start()
      }
    }
  };
}
impl_ctx_input!(ActionInput<'text, ActionState>);
impl_ctx_input!(&ActionInput<'text, ActionState>);
impl_ctx_input!(&mut ActionInput<'text, ActionState>);

/// This trait is used to unify [`ActionOutput`],
/// `&ActionOutput` and `&mut ActionOutput` in
/// [`AcceptedActionDecoratorContext`]
pub trait AcceptedActionDecoratorContextOutput {
  /// Don't use this. This is only used internally.
  fn _digested(&self) -> usize;
}
macro_rules! impl_ctx_output {
  ($t:ty) => {
    impl<Kind, OptionErrorType> AcceptedActionDecoratorContextOutput for $t {
      fn _digested(&self) -> usize {
        self.digested
      }
    }
  };
}
impl_ctx_output!(ActionOutput<Kind, OptionErrorType>);
impl_ctx_output!(&ActionOutput<Kind, OptionErrorType>);
impl_ctx_output!(&mut ActionOutput<Kind, OptionErrorType>);

// these methods are available no matter if `InputType` is
// `ActionInput`, `&ActionInput` or `&mut ActionInput`,
// and no matter the `OutputType` is
// `ActionOutput`, `&ActionOutput` or `&mut ActionOutput`
impl<
    'text,
    InputType: AcceptedActionDecoratorContextInput<'text>,
    OutputType: AcceptedActionDecoratorContextOutput,
  > AcceptedActionOutputContext<InputType, OutputType>
{
  /// The [`Range::end`](crate::lexer::token::Range) of the token that this action will emit.
  pub fn end(&self) -> usize {
    self.input._start() + self.output._digested()
  }

  /// The [`content`](crate::lexer::token::Token::content) of the token that this action will emit.
  pub fn content(&self) -> &'text str {
    // we don't cache this slice since it might not be used frequently
    &self.input._text()[self.input._start()..self.end()]
  }

  /// The rest of the input text after this action is accepted.
  pub fn rest(&self) -> &'text str {
    // we don't cache this slice since it might not be used frequently
    &self.input._text()[self.end()..]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_input() -> ActionInput<'static, ()> {
    ActionInput::new("123", 1, ()).unwrap()
  }
  fn create_output() -> ActionOutput<(), Option<()>> {
    ActionOutput {
      kind: (),
      digested: 1,
      muted: false,
      error: None::<()>,
    }
  }

  #[test]
  fn accepted_action_decorator_context() {
    // ensure the methods are working

    // input and output
    AcceptedActionOutputContext {
      input: create_input(),
      output: create_output(),
    }
    .end();

    // &input and output
    AcceptedActionOutputContext {
      input: &create_input(),
      output: create_output(),
    }
    .end();

    // &mut input and output
    AcceptedActionOutputContext {
      input: &mut create_input(),
      output: create_output(),
    }
    .end();

    // input and &output
    AcceptedActionOutputContext {
      input: create_input(),
      output: &create_output(),
    }
    .end();

    // &input and &output
    AcceptedActionOutputContext {
      input: &create_input(),
      output: &create_output(),
    }
    .end();

    // &mut input and &output
    AcceptedActionOutputContext {
      input: &mut create_input(),
      output: &create_output(),
    }
    .end();

    // input and &mut output
    AcceptedActionOutputContext {
      input: create_input(),
      output: &mut create_output(),
    }
    .end();

    // &input and &mut output
    AcceptedActionOutputContext {
      input: &create_input(),
      output: &mut create_output(),
    }
    .end();

    // &mut input and &mut output
    AcceptedActionOutputContext {
      input: &mut create_input(),
      output: &mut create_output(),
    }
    .end();

    // ensure the value is correct
    assert_eq!(
      AcceptedActionOutputContext {
        input: create_input(),
        output: create_output(),
      }
      .end(),
      2
    );
    assert_eq!(
      AcceptedActionOutputContext {
        input: create_input(),
        output: create_output(),
      }
      .content(),
      "2"
    );
    assert_eq!(
      AcceptedActionOutputContext {
        input: create_input(),
        output: create_output(),
      }
      .rest(),
      "3"
    );
  }
}
