use super::input::ActionInput;
use std::ops::{Deref, DerefMut};

pub struct ActionOutput<Kind, ErrorType> {
  pub kind: Kind,
  /// How many characters are digested by this action.
  pub digested: usize,
  /// If `true`, the action is accepted but no token is emitted,
  /// and the lexing process will continue.
  pub muted: bool,
  /// If `Some`, the action is still accepted,
  /// and error tokens will be collected in
  /// [`LexOutput::errors`](crate::lexer::output::LexOutput::errors).
  pub error: Option<ErrorType>,
}

pub struct ActionOutputWithoutKind<ErrorType> {
  /// See [`ActionOutput::digested`].
  pub digested: usize,
  /// See [`ActionOutput::muted`].
  pub muted: bool,
  /// See [`ActionOutput::error`].
  pub error: Option<ErrorType>,
}

impl<ErrorType> Into<ActionOutput<(), ErrorType>> for ActionOutputWithoutKind<ErrorType> {
  fn into(self) -> ActionOutput<(), ErrorType> {
    ActionOutput {
      kind: (),
      digested: self.digested,
      muted: self.muted,
      error: self.error,
    }
  }
}

pub struct EnhancedActionOutput<'buffer, Kind, ErrorType> {
  /// The original [`ActionOutput`].
  pub raw: ActionOutput<Kind, ErrorType>,
  /// The [`ActionInput::buffer`].
  pub buffer: &'buffer str,
  /// The [`ActionInput::start`].
  pub start: usize,
}

impl<'buffer, Kind, ErrorType> Deref for EnhancedActionOutput<'buffer, Kind, ErrorType> {
  type Target = ActionOutput<Kind, ErrorType>;

  fn deref(&self) -> &Self::Target {
    &self.raw
  }
}

impl<'buffer, Kind, ErrorType> DerefMut for EnhancedActionOutput<'buffer, Kind, ErrorType> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.raw
  }
}

impl<'buffer, Kind, ErrorType> EnhancedActionOutput<'buffer, Kind, ErrorType> {
  pub fn new<ActionState>(
    input: &ActionInput<'buffer, '_, ActionState>,
    output: ActionOutput<Kind, ErrorType>,
  ) -> Self {
    EnhancedActionOutput {
      raw: output,
      start: input.start(),
      buffer: input.buffer(),
    }
  }

  /// The [`Range::end`](crate::lexer::token::Range::end) of the token that this action will emit.
  pub fn end(&self) -> usize {
    self.start + self.digested
  }

  /// The content of the token that this action will emit.
  pub fn content(&self) -> &'buffer str {
    &self.buffer[self.start..self.end()]
  }

  /// The rest of the input text after this action is accepted.
  pub fn rest(&self) -> &'buffer str {
    &self.buffer[self.end()..]
  }
}

impl<'buffer, Kind, ErrorType> Into<ActionOutput<Kind, ErrorType>>
  for EnhancedActionOutput<'buffer, Kind, ErrorType>
{
  fn into(self) -> ActionOutput<Kind, ErrorType> {
    self.raw
  }
}

impl<'buffer, Kind, ErrorType> Into<Option<ActionOutput<Kind, ErrorType>>>
  for EnhancedActionOutput<'buffer, Kind, ErrorType>
{
  fn into(self) -> Option<ActionOutput<Kind, ErrorType>> {
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
    let output: ActionOutput<(), ()> = output.into();
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
    let output = EnhancedActionOutput::<(), ()>::new(&input, output.into());

    // ensure we can deref and deref_mut
    assert_eq!(output.digested, 2);
    assert_eq!(output.muted, false);
    assert_eq!(output.error, None);
    assert!(matches!(output.kind, ()));

    // access fields from input
    assert_eq!(output.start, 1);
    assert_eq!(output.buffer, "123");

    // helpers
    assert_eq!(output.end(), 3);
    assert_eq!(output.content(), "23");
    assert_eq!(output.rest(), "");
  }
}
