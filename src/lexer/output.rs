use super::options::ReLexContext;

pub struct PeekOutput<TokenType, ActionState> {
  pub token: Option<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
  pub action_state: ActionState,
}

pub struct LexOutput<TokenType, ReLexType> {
  pub token: Option<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
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

pub struct TrimOutput<TokenType> {
  pub digested: usize,
  pub errors: Vec<TokenType>,
}
