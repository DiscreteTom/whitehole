use super::input::ActionInput;
use std::ops::{Deref, DerefMut};

pub struct ActionOutput<Kind, ErrorType> {
  pub kind: Kind,
  /// How many characters are digested by this action.
  pub digested: usize,
  /// If `true`, the action is accepted but no token is emitted,
  /// and the lexing process will continue.
  pub muted: bool,
  pub error: Option<ErrorType>,
}

pub struct ActionOutputWithoutKind<ErrorType> {
  pub digested: usize,
  pub muted: bool,
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

  pub fn end(&self) -> usize {
    self.start + self.digested
  }

  pub fn content(&self) -> &'buffer str {
    &self.buffer[self.start..self.end()]
  }

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
