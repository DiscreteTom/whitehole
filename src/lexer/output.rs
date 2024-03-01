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

// this should never be constructed by user
// and the fields should never be accessed by user
// because the `action_index` is an internal index
#[derive(Default, Clone, Debug)]
pub struct ReLexContext {
  /// From which action to re-lex.
  /// This is effective only if
  /// the [`ActionInput::start`](crate::lexer::action::input::ActionInput::start)
  /// equals to `self.start`.
  pub(crate) action_index: usize,
  pub(crate) start: usize,
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
