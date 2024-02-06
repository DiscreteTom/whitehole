use crate::lexer::token::{TokenKind, TokenKindId};

// TODO: NTKind should impl another trait instead of TokenKind
pub enum GrammarKind<Kind: TokenKind> {
  T(Kind),
  NT(Kind),
}

impl<Kind: TokenKind> GrammarKind<Kind> {
  pub fn id(&self) -> TokenKindId {
    match self {
      GrammarKind::T(kind) => kind.id(),
      GrammarKind::NT(kind) => kind.id(),
    }
  }
}

pub struct Grammar<Kind: TokenKind> {
  kind: GrammarKind<Kind>,
  text: Option<String>,
}

impl<Kind: TokenKind> Grammar<Kind> {
  /// Should only be called by the grammar repo.
  pub fn new(kind: GrammarKind<Kind>, text: Option<String>) -> Self {
    Self { kind, text }
  }

  pub fn kind(&self) -> &GrammarKind<Kind> {
    &self.kind
  }
  pub fn text(&self) -> &Option<String> {
    &self.text
  }
}
