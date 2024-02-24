use crate::lexer::{token::TokenKind, trimmed::TrimmedLexer};

pub type LexerPanicHandler<TKind, LexerActionState, LexerErrorType> = Box<
  dyn Fn(
    TrimmedLexer<TKind, LexerActionState, LexerErrorType>,
  ) -> TrimmedLexer<TKind, LexerActionState, LexerErrorType>,
>;

/// Take one char from the rest of the buffer and reset lexer's action state.
pub fn default_lexer_panic_handler<
  'buffer,
  TKind: TokenKind<TKind> + 'static,
  LexerActionState: Clone + Default + 'static,
  LexerErrorType: 'static,
>(
  lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
) -> TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType> {
  lexer.take(1, None).into()
}
