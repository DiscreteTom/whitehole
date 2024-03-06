use super::input::ActionInput;
use std::ops::{Deref, DerefMut};

pub struct ActionOutput<Kind, OptionErrorType> {
  pub kind: Kind,
  /// How many characters are digested by this action.
  /// `0` is allowed, but be careful with infinite loop.
  pub digested: usize,
  /// If `true`, the action is accepted but no token is emitted,
  /// and the lexing process will continue.
  pub muted: bool,
  /// If `Some`, the action is still accepted,
  /// and error tokens will be collected in
  /// [`LexOutput::errors`](crate::lexer::output::LexOutput::errors).
  pub error: OptionErrorType, // this will be `Option<ErrorType>` or `&Option<ErrorType>`
}

pub struct ActionOutputWithoutKind<OptionErrorType> {
  /// See [`ActionOutput::digested`].
  pub digested: usize,
  /// See [`ActionOutput::muted`].
  pub muted: bool,
  /// See [`ActionOutput::error`].
  pub error: OptionErrorType,
}

impl<OptionErrorType> Into<ActionOutput<(), OptionErrorType>>
  for ActionOutputWithoutKind<OptionErrorType>
{
  fn into(self) -> ActionOutput<(), OptionErrorType> {
    ActionOutput {
      kind: (),
      digested: self.digested,
      muted: self.muted,
      error: self.error,
    }
  }
}

pub struct EnhancedActionOutput<'text, Kind, OptionErrorType> {
  /// The original [`ActionOutput`].
  pub raw: ActionOutput<Kind, OptionErrorType>,
  /// The [`ActionInput::text`].
  pub text: &'text str,
  /// The [`ActionInput::start`].
  pub start: usize,
}

impl<'text, Kind, OptionErrorType> Deref for EnhancedActionOutput<'text, Kind, OptionErrorType> {
  type Target = ActionOutput<Kind, OptionErrorType>;

  fn deref(&self) -> &Self::Target {
    &self.raw
  }
}

impl<'text, Kind, OptionErrorType> DerefMut for EnhancedActionOutput<'text, Kind, OptionErrorType> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.raw
  }
}

impl<'text, Kind, OptionErrorType> EnhancedActionOutput<'text, Kind, OptionErrorType> {
  pub fn new<ActionState>(
    input: &ActionInput<'text, ActionState>,
    output: ActionOutput<Kind, OptionErrorType>,
  ) -> Self {
    EnhancedActionOutput {
      raw: output,
      start: input.start(),
      text: input.text(),
    }
  }

  /// The [`Range::end`](crate::lexer::token::Range::end) of the token that this action will emit.
  pub fn end(&self) -> usize {
    self.start + self.digested
  }

  /// The content of the token that this action will emit.
  pub fn content(&self) -> &'text str {
    &self.text[self.start..self.end()]
  }

  /// The rest of the input text after this action is accepted.
  pub fn rest(&self) -> &'text str {
    &self.text[self.end()..]
  }
}

impl<'text, Kind, OptionErrorType> Into<ActionOutput<Kind, OptionErrorType>>
  for EnhancedActionOutput<'text, Kind, OptionErrorType>
{
  fn into(self) -> ActionOutput<Kind, OptionErrorType> {
    self.raw
  }
}

impl<'text, Kind, OptionErrorType> Into<Option<ActionOutput<Kind, OptionErrorType>>>
  for EnhancedActionOutput<'text, Kind, OptionErrorType>
{
  fn into(self) -> Option<ActionOutput<Kind, OptionErrorType>> {
    Some(self.into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn action_output_without_kind() {
    let output = ActionOutputWithoutKind {
      digested: 2,
      muted: false,
      error: None,
    };
    let output: ActionOutput<(), Option<()>> = output.into();
    assert_eq!(output.kind, ());
    assert_eq!(output.digested, 2);
    assert_eq!(output.muted, false);
    assert_eq!(output.error, None);
  }

  #[test]
  fn enhanced_action_output() {
    let mut state = ();
    let input = ActionInput::new("123", 1, &mut state);
    let output = ActionOutputWithoutKind {
      digested: 2,
      muted: false,
      error: None,
    };
    let output = EnhancedActionOutput::<(), Option<()>>::new(&input, output.into());

    // ensure we can deref and deref_mut
    assert_eq!(output.digested, 2);
    assert_eq!(output.muted, false);
    assert_eq!(output.error, None);
    assert!(matches!(output.kind, ()));

    // access fields from input
    assert_eq!(output.start, 1);
    assert_eq!(output.text, "123");

    // helpers
    assert_eq!(output.end(), 3);
    assert_eq!(output.content(), "23");
    assert_eq!(output.rest(), "");
  }
}
