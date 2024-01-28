use super::candidate::Candidate;
use crate::lexer::token::TokenKind;

pub struct State<'gr, 'grammar, 'candidate, TKind: TokenKind, NTKind> {
  candidates: Vec<&'candidate Candidate<'grammar, 'gr, TKind, NTKind>>,
}
