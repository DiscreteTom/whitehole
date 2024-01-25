pub struct PeekOutput<TokenType, ActionState> {
  pub token: Option<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
  pub action_state: ActionState,
}

pub struct LexOutput<TokenType> {
  pub token: Option<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
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

pub struct IntoTrimmedOutput<TokenType, TrimmedLexer> {
  pub digested: usize,
  pub errors: Vec<TokenType>,
  pub trimmed_lexer: TrimmedLexer,
}
