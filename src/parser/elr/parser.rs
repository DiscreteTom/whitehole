use super::dfa::dfa::Dfa;
use crate::lexer::token::TokenKind;

pub struct Parser<
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind> + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  dfa: Dfa<TKind, NTKind, ASTData, ErrorType, Global>,
}
