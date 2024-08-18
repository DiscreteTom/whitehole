#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionOutput<BindingType, OptionErrorType> {
  /// The [`Token::binding`](crate::lexer::token::Token::binding)
  /// that is created by this action.
  /// This is often [`TokenKindIdBinding<Kind>`](crate::lexer::token::TokenKindIdBinding)
  /// but might be `&TokenKindIdBinding<Kind>`
  /// in some action decorators' contexts.
  pub binding: BindingType,
  /// How many bytes are digested by this action.
  /// # Caveats
  /// `0` is allowed, but be careful with infinite loops.
  ///
  /// The caller MUST ensure this value is smaller than the length of
  /// [`ActionInput::rest`](crate::lexer::action::input::ActionInput::rest).
  pub digested: usize,
  /// If [`Some`], the action is still accepted (not rejected),
  /// and the error will be collected by
  /// [`LexOptions::errors`](crate::lexer::options::LexOptions::errors).
  ///
  /// This is often [`Option<ErrorType>`] but might be `&Option<ErrorType>`
  /// in some action decorators' contexts.
  pub error: OptionErrorType,
}
