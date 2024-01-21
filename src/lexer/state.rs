// use super::token::Token;
// use std::rc::Rc;

// pub struct LexerState<Kind: 'static, ErrorType: 'static> {
//   // use Rc to lazy-clone the buffer
//   // so that every `lexer.clone` won't clone the buffer
//   // only when the buffer is modified, it will be cloned
//   buffer: Rc<String>,
//   digested: usize,
//   line_indexes: Vec<usize>,
//   trimmed: bool,
//   errors: Vec<Rc<Token<Kind, ErrorType>>>,
// }
