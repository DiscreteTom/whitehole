pub struct LexOutput<TokenType, ReLexType> {
  pub token: Option<TokenType>,
  /// How many bytes are digested during the whole lexing loop in current lexing.
  /// This is NOT [`ActionOutput::digested`](crate::lexer::action::ActionOutput::digested)
  /// because there might be many actions which are accepted during multiple iterations
  /// of the lexing loop, this value is the sum of them.
  pub digested: usize,
  /// Muted error tokens during this lex.
  /// # Caveat
  /// [`Self::token`] will NOT be included in this
  /// even if it's an error token.
  pub errors: Vec<TokenType>, // [[@muted error tokens are also collected]]
  /// If [`Some`], the lex is re-lex-able and you can use this value
  /// to continue a lex. This *might* be [`Some`] only if the [`LexOptions::fork`](super::options::LexOptions::fork)
  /// is enabled.
  pub re_lex: ReLexType, // TODO: example
}
