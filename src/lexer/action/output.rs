#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionOutput<BindingType> {
  /// The [`Token::binding`](crate::lexer::token::Token::binding)
  /// that is created by this action.
  /// This is often [`TokenKindIdBinding<Kind>`](crate::lexer::token::TokenKindIdBinding)
  /// but might be `&TokenKindIdBinding<Kind>`
  /// in some action decorators' contexts.
  pub binding: BindingType, // TODO: change type to TokenKindIdBinding
  /// How many bytes are digested by this action.
  /// # Caveats
  /// `0` is allowed, but be careful with infinite loops.
  ///
  /// The caller MUST ensure this value is smaller than the length of
  /// [`ActionInput::rest`](crate::lexer::action::input::ActionInput::rest).
  pub digested: usize,
}
