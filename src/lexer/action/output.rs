#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionOutput<Kind, OptionErrorType> {
  /// The [`Token::kind`](crate::lexer::token::Token::kind)
  /// that is created by this action.
  pub kind: Kind,
  /// How many bytes are digested by this action.
  /// `0` is allowed, but be careful with infinite loops.
  pub digested: usize,
  /// If [`Some`], the action is still accepted,
  /// and the error will be collected by
  /// [`LexOutput::errors`](crate::lexer::output::LexOutput::errors).
  pub error: OptionErrorType, // this will be `Option<ErrorType>` or `&Option<ErrorType>`
}
