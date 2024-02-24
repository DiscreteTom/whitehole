use crate::lexer::{token::TokenKind, trimmed::TrimmedLexer};

pub type LexerPanicHandler<TKind, LexerActionState, LexerErrorType> =
  Box<dyn Fn(&mut TrimmedLexer<TKind, LexerActionState, LexerErrorType>)>;

/// Take one char from the rest of the buffer and reset lexer's action state.
pub fn default_lexer_panic_handler<
  'buffer,
  TKind: TokenKind<TKind> + 'static,
  LexerActionState: Clone + Default + 'static,
  LexerErrorType: 'static,
>(
  lexer: &mut TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
) {
  lexer.take_and_trim(1, None);
}
