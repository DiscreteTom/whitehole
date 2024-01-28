use crate::lexer::token::TokenKind;

pub enum GrammarKind<TKind: TokenKind, NTKind> {
  T(TKind),
  NT(NTKind),
}

pub struct Grammar<TKind: TokenKind, NTKind> {
  kind: GrammarKind<TKind, NTKind>,
  text: Option<String>,
}

impl<TKind: TokenKind, NTKind> Grammar<TKind, NTKind> {
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
