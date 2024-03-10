use super::options::ReLexContext;

pub struct LexOutput<TokenType, ReLexType> {
  pub token: Option<TokenType>,
  pub digested: usize,
  /// Error tokens during this lex.
  /// Muted error tokens are included in this list.
  /// [`LexOutput::token`] will not be included in this list
  /// even if it's an error token.
  pub errors: Vec<TokenType>, // [[@muted error tokens are also collected]]
  /// This will always be `None`
  /// unless you set [`LexOptions::fork`](crate::lexer::options::LexOptions::fork) to `true`.
  /// If `Some`, the lex is re-lex-able.
  pub re_lex: Option<ReLexType>,
}

pub struct ReLexable<LexerType> {
  pub lexer: LexerType,
  pub context: ReLexContext,
}

pub struct LexAllOutput<TokenType> {
  pub tokens: Vec<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
}
