use crate::lexer::token::TokenKind;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct GrammarId(pub usize);

pub enum GrammarKind<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> {
  T(TKind),
  NT(NTKind),
}

pub struct Grammar<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> {
  id: GrammarId,
  kind: GrammarKind<TKind, NTKind>,
  text: Option<String>,
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> Grammar<TKind, NTKind> {
  /// Should only be called by the grammar repo.
  pub fn new(id: GrammarId, kind: GrammarKind<TKind, NTKind>, text: Option<String>) -> Self {
    Self { id, kind, text }
  }

  pub fn id(&self) -> &GrammarId {
    &self.id
  }
  pub fn kind(&self) -> &GrammarKind<TKind, NTKind> {
    &self.kind
  }
  pub fn text(&self) -> &Option<String> {
    &self.text
  }
}
