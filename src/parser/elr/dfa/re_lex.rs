use super::stack::Stack;

pub struct ReLexState<StateType, LexerType> {
  pub buffer_len: usize,
  pub state_stack: Stack<StateType>,
  pub reducing_stack: Vec<usize>,
  pub lexer: LexerType,
  pub errors_len: usize,
  // TODO: do we need `next_token` and `need_lex`?
}
