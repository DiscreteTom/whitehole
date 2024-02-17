use crate::lexer::token::TokenKind;
use std::hash::Hash;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct GrammarId(pub usize);

#[derive(Clone)]
pub enum GrammarKind<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> {
  T(TKind),
  NT(NTKind),
  Literal(String),
}

pub struct Grammar<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> {
  id: GrammarId,
  kind: GrammarKind<TKind, NTKind>,
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> Hash for Grammar<TKind, NTKind> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
  }
}
impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> PartialEq for Grammar<TKind, NTKind> {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}
impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> Eq for Grammar<TKind, NTKind> {}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> Grammar<TKind, NTKind> {
  /// Should only be called by the grammar repo.
  pub fn new(id: GrammarId, kind: GrammarKind<TKind, NTKind>) -> Self {
    Self { id, kind }
  }

  pub fn id(&self) -> &GrammarId {
    &self.id
  }
  pub fn kind(&self) -> &GrammarKind<TKind, NTKind> {
    &self.kind
  }
}
