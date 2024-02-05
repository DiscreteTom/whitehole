use crate::lexer::token::{TokenKind, TokenKindId};

// TODO: NTKind should impl another trait instead of TokenKind
pub enum GrammarKind<TKind: TokenKind, NTKind: TokenKind> {
  T(TKind),
  NT(NTKind),
}

impl<TKind: TokenKind, NTKind: TokenKind> GrammarKind<TKind, NTKind> {
  pub fn id(&self) -> TokenKindId {
    match self {
      GrammarKind::T(kind) => kind.id(),
      GrammarKind::NT(kind) => kind.id(),
    }
  }
}

pub struct Grammar<TKind: TokenKind, NTKind: TokenKind> {
  kind: GrammarKind<TKind, NTKind>,
  text: Option<String>,
}

impl<TKind: TokenKind, NTKind: TokenKind> Grammar<TKind, NTKind> {
  /// Should only be called by the grammar repo.
  pub fn new(kind: GrammarKind<TKind, NTKind>, text: Option<String>) -> Self {
    Self { kind, text }
  }

  pub fn kind(&self) -> &GrammarKind<TKind, NTKind> {
    &self.kind
  }
  pub fn text(&self) -> &Option<String> {
    &self.text
  }
}
