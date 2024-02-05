use super::grammar::{Grammar, GrammarKind};
use crate::lexer::token::TokenKind;
use std::collections::HashMap;

pub type GrammarId = String;

pub struct GrammarRepo<TKind: TokenKind, NTKind: TokenKind> {
  map: HashMap<GrammarId, Grammar<TKind, NTKind>>,
}

impl<TKind: TokenKind, NTKind: TokenKind> Default for GrammarRepo<TKind, NTKind> {
  fn default() -> Self {
    Self {
      map: HashMap::new(),
    }
  }
}

impl<TKind: TokenKind, NTKind: TokenKind> GrammarRepo<TKind, NTKind> {
  pub fn get_or_create_t(&mut self, id: GrammarId, kind: TKind) -> &Grammar<TKind, NTKind> {
    self
      .map
      .entry(id)
      .or_insert(Grammar::new(GrammarKind::T(kind), None))
  }

  pub fn get_or_create_nt(&mut self, id: GrammarId, kind: NTKind) -> &Grammar<TKind, NTKind> {
    self
      .map
      .entry(id)
      .or_insert(Grammar::new(GrammarKind::NT(kind), None))
  }

  pub fn get_or_create_literal(
    &mut self,
    id: GrammarId,
    kind: TKind,
    text: String,
  ) -> &Grammar<TKind, NTKind> {
    self
      .map
      .entry(id)
      .or_insert(Grammar::new(GrammarKind::T(kind), Some(text)))
  }
}
