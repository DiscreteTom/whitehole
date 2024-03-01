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
  /// If `Some`, the lex is re-lex-able.
  pub re_lex: Option<ReLexType>,
}

#[derive(Default, Clone)]
pub struct ReLexActionIndex(pub usize);

pub struct ReLexContext<LexerType> {
  pub action_index: ReLexActionIndex,
  pub lexer: LexerType,
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
