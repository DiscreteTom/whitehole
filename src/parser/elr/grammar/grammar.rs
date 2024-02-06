use crate::lexer::token::TokenKind;

pub enum GrammarType {
  T,
  NT,
}

pub type GrammarId = usize;

pub struct Grammar<Kind: TokenKind> {
  kind: Kind,
  grammar_type: GrammarType,
  text: Option<String>,
  id: GrammarId,
}

impl<Kind: TokenKind> Grammar<Kind> {
  /// Should only be called by the grammar repo.
  pub fn new(grammar_type: GrammarType, kind: Kind, text: Option<String>, id: GrammarId) -> Self {
    Self {
      grammar_type,
      kind,
      text,
      id,
    }
  }

  pub fn kind(&self) -> &Kind {
    &self.kind
  }
  pub fn grammar_type(&self) -> &GrammarType {
    &self.grammar_type
  }
  pub fn text(&self) -> &Option<String> {
    &self.text
  }
  pub fn id(&self) -> GrammarId {
    self.id
  }
}
