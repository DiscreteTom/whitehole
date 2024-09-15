use crate::kind::KindIdBinding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionOutput<Kind> {
  /// The [`Token::binding`](crate::lexer::token::Token::binding)
  /// that is created by this action.
  pub binding: KindIdBinding<Kind>,
  /// How many bytes are digested by this action.
  /// # Caveats
  /// `0` is allowed, but be careful with infinite loops.
  ///
  /// The caller MUST ensure this value is smaller than the length of
  /// [`ActionInput::rest`](crate::lexer::action::input::ActionInput::rest).
  pub digested: usize,
}
