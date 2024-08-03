#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexOutput<TokenType, ErrAcc, ReLexableType> {
  /// If all actions are rejected, this will be [`None`].
  pub token: Option<TokenType>,
  /// How many bytes are digested in this lexing.
  /// # Caveats
  /// This might not be `0` even [`Self::token`] is [`None`]
  /// because there might be many actions which are accepted during multiple iterations
  /// of the lexing loop, this value is the total digested bytes in one lexing.
  pub digested: usize,
  /// If there are any errors during the lexing, they will be accumulated here.
  /// See [`LexOptions::errors_to`](super::options::LexOptions::errors_to) for more information.
  pub errors: ErrAcc,
  /// If re-lex is disabled, this will always be `()`.
  ///
  /// This *might* be [`Some`] only if the [`LexOptions::fork`](super::options::LexOptions::fork)
  /// is enabled.
  /// If [`Some`], the lex is re-lexable and you can use this value
  /// to continue a lex.
  ///
  /// See [`ReLexContext`](crate::lexer::re_lex::ReLexContext) for more information.
  pub re_lexable: ReLexableType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimOutput<ErrAcc> {
  /// How many bytes are digested in this lexing.
  pub digested: usize,
  /// If there are any errors during the lexing, they will be accumulated here.
  pub errors: ErrAcc,
}
