pub struct ActionOutput<Kind, OptionErrorType> {
  pub kind: Kind,
  /// How many bytes are digested by this action.
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
