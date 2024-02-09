use crate::lexer::token::{TokenKind, TokenKindId};

pub type GrammarId = usize;

pub enum GrammarKind<TKind: TokenKind, NTKind: TokenKind> {
  T(TKind),
  NT(NTKind),
}

impl<TKind: TokenKind, NTKind: TokenKind> TokenKind for GrammarKind<TKind, NTKind> {
  fn id(&self) -> TokenKindId {
    match self {
      GrammarKind::T(kind) => kind.id(),
      GrammarKind::NT(kind) => kind.id(),
    }
  }
}

pub struct Grammar<TKind: TokenKind, NTKind: TokenKind> {
  id: GrammarId,
  kind: GrammarKind<TKind, NTKind>,
  text: Option<String>,
}

impl<TKind: TokenKind, NTKind: TokenKind> Grammar<TKind, NTKind> {
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
