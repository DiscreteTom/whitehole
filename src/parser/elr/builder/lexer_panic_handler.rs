use crate::lexer::{token::TokenKind, Lexer};
use std::rc::Rc;

pub type LexerPanicHandler<TKind, LexerActionState, LexerErrorType> =
  Rc<dyn Fn(&mut Lexer<TKind, LexerActionState, LexerErrorType>)>;

/// Take one char from the rest of the buffer and reset lexer's action state.
pub fn default_lexer_panic_handler<
  'buffer,
  TKind: TokenKind<TKind> + 'static,
  LexerActionState: Clone + Default + 'static,
  LexerErrorType: 'static,
>(
  lexer: &mut Lexer<'buffer, TKind, LexerActionState, LexerErrorType>,
) {
  lexer.take(1);
}
