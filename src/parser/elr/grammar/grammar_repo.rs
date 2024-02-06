use super::grammar::{Grammar, GrammarKind};
use crate::lexer::token::TokenKind;
use std::collections::HashMap;

pub type GrammarId = String;

pub struct GrammarRepo<Kind: TokenKind> {
  map: HashMap<GrammarId, Grammar<Kind>>,
}

impl<Kind: TokenKind> Default for GrammarRepo<Kind> {
  fn default() -> Self {
    Self {
      map: HashMap::new(),
    }
  }
}

impl<Kind: TokenKind> GrammarRepo<Kind> {
  pub fn get_or_create_t(&mut self, id: GrammarId, kind: Kind) -> &Grammar<Kind> {
    self
      .map
      .entry(id)
      .or_insert(Grammar::new(GrammarKind::T(kind), None))
  }

  pub fn get_or_create_nt(&mut self, id: GrammarId, kind: Kind) -> &Grammar<Kind> {
    self
      .map
      .entry(id)
      .or_insert(Grammar::new(GrammarKind::NT(kind), None))
  }

  pub fn get_or_create_literal(
    &mut self,
    id: GrammarId,
    kind: Kind,
    text: String,
  ) -> &Grammar<Kind> {
    self
      .map
      .entry(id)
      .or_insert(Grammar::new(GrammarKind::T(kind), Some(text)))
  }
}
