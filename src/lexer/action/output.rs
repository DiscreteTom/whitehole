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
  /// If [`Some`], the action is still accepted,
  /// and a token will be created even if the output is muted.
  /// Muted error tokens will be collected in
  /// [`LexOutput::errors`](crate::lexer::output::LexOutput::errors).
  pub error: OptionErrorType, // this will be `Option<ErrorType>` or `&Option<ErrorType>`
}

impl<Kind, OptionErrorType> ActionOutput<Kind, OptionErrorType> {
  /// Enhance the original [`ActionOutput`] with
  /// [`start`](EnhancedActionOutput::start), [`text`](EnhancedActionOutput::text),
  /// [`end`](EnhancedActionOutput::end), [`content`](EnhancedActionOutput::content)
  /// and [`rest`](EnhancedActionOutput::rest).
  pub fn into_enhanced<'text, ActionState>(
    self,
    input: &ActionInput<'text, '_, ActionState>,
  ) -> EnhancedActionOutput<'text, Kind, OptionErrorType> {
    EnhancedActionOutput {
      base: self,
      start: input.start(),
      text: input.text(),
    }
  }
}

/// See [`ActionOutput::into_enhanced`].
pub struct EnhancedActionOutput<'text, Kind, OptionErrorType> {
  /// The original [`ActionOutput`].
  pub base: ActionOutput<Kind, OptionErrorType>,
  /// The [`ActionInput::text`].
  pub text: &'text str,
  /// The [`ActionInput::start`].
  pub start: usize,
}

impl<'text, Kind, OptionErrorType> Deref for EnhancedActionOutput<'text, Kind, OptionErrorType> {
  type Target = ActionOutput<Kind, OptionErrorType>;
  fn deref(&self) -> &Self::Target {
    &self.base
  }
}

impl<'text, Kind, OptionErrorType> DerefMut for EnhancedActionOutput<'text, Kind, OptionErrorType> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.base
  }
}

impl<'text, 'action_state, Kind, OptionErrorType>
  EnhancedActionOutput<'text, Kind, OptionErrorType>
{
  /// The [`Range::end`](crate::lexer::token::Range) of the token that this action will emit.
  pub fn end(&self) -> usize {
    self.start + self.digested
  }

  /// The [`content`](crate::lexer::token::Token::content) of the token that this action will emit.
  pub fn content(&self) -> &'text str {
    // we don't cache this slice since it might not be used frequently
    &self.text[self.start..self.end()]
  }

  /// The rest of the input text after this action is accepted.
  pub fn rest(&self) -> &'text str {
    // we don't cache this slice since it might not be used frequently
    &self.text[self.end()..]
  }
}

impl<'text, Kind, OptionErrorType> Into<ActionOutput<Kind, OptionErrorType>>
  for EnhancedActionOutput<'text, Kind, OptionErrorType>
{
  fn into(self) -> ActionOutput<Kind, OptionErrorType> {
    self.base
  }
}

// impl this so that we can just return `enhanced_output.into()` in action's exec
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
  use crate::lexer::token::{MockTokenKind, TokenKindId, TokenKindIdProvider};

  #[test]
  fn enhanced_action_output() {
    let mut state = ();
    let input = ActionInput::new("123", 1, &mut state);
    let output = ActionOutput {
      kind: MockTokenKind::new(()),
      digested: 2,
      muted: false,
      error: None::<()>,
    };
    let output = output.into_enhanced(&input);

    // ensure we can deref and deref_mut
    assert_eq!(output.digested, 2);
    assert_eq!(output.muted, false);
    assert_eq!(output.error, None);
    assert!(matches!(output.kind.data, ()));
    assert_eq!(output.kind.id(), &TokenKindId::new(0));

    // access fields from input
    assert_eq!(output.start, 1);
    assert_eq!(output.text, "123");

    // helpers
    assert_eq!(output.end(), 3);
    assert_eq!(output.content(), "23");
    assert_eq!(output.rest(), "");
  }
}
