#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionOutput<BindingType, OptionErrorType> {
  /// The [`Token::binding`](crate::lexer::token::Token::binding)
  /// that is created by this action.
  pub binding: BindingType,
  /// How many bytes are digested by this action.
  /// `0` is allowed, but be careful with infinite loops.
  pub digested: usize,
  /// If [`Some`], the action is still accepted,
  /// and the error will be collected by
  /// [`LexOutput::errors`](crate::lexer::output::LexOutput::errors).
  pub error: OptionErrorType, // this will be `Option<ErrorType>` or `&Option<ErrorType>`
}
