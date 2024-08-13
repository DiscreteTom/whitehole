#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexOutput<TokenType, ForkOutputType> {
  /// If all actions are rejected, this will be [`None`].
  pub token: Option<TokenType>,
  /// How many bytes are digested in this lexing.
  /// # Caveats
  /// This might not be `0` even [`Self::token`] is [`None`]
  /// because there might be many actions which are accepted during multiple iterations
  /// of the lexing loop, this value is the total digested bytes in one lexing.
  pub digested: usize,
  /// If fork is disabled, this will always be `()`.
  ///
  /// This *might* be [`Some`] only if the [`LexOptions::fork`](super::options::LexOptions::fork)
  /// is enabled.
  /// If [`Some`], the lex is re-lexable and you can use this value
  /// to continue a lex.
  ///
  /// See [`ReLexContext`](crate::lexer::re_lex::ReLexContext) for more information.
  pub fork: ForkOutputType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimOutput {
  /// How many bytes are digested in this lexing.
  pub digested: usize,
}
