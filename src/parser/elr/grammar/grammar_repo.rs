use super::grammar::{Grammar, GrammarId, GrammarType};
use crate::lexer::token::{TokenKind, TokenKindId};
use std::collections::{hash_map::Entry, HashMap};

pub struct GrammarRepo<Kind: TokenKind> {
  /// This is used to check if a T grammar is already created.
  t_cache: HashMap<TokenKindId, HashMap<Option<String>, GrammarId>>,
  /// This is used to check if a NT grammar is already created.
  nt_cache: HashMap<TokenKindId, GrammarId>,
  /// This is used to get the grammar by id.
  // TODO: is this needed? can we just store grammar in caches?
  map: HashMap<GrammarId, Grammar<Kind>>,
}

impl<Kind: TokenKind> Default for GrammarRepo<Kind> {
  fn default() -> Self {
    Self {
      t_cache: HashMap::new(),
      nt_cache: HashMap::new(),
      map: HashMap::new(),
    }
  }
}

impl<Kind: TokenKind> GrammarRepo<Kind> {
  pub fn get_or_create_t(&mut self, kind: Kind) -> &Grammar<Kind> {
    match self
      .t_cache
      .entry(kind.id())
      .or_insert(HashMap::new())
      .entry(None) // None for no text field
    {
      Entry::Occupied(o) => &self.map[&o.get()],
      Entry::Vacant(v) => {
        let id = self.map.len();
        v.insert(id);
        match self.map.entry(id) {
          // this should never happen
          Entry::Occupied(_) => panic!("Grammar with id {} already exists", id),
          Entry::Vacant(v) => v.insert(Grammar::new(GrammarType::T, kind, None, id)),
        }
      }
    }
  }

  pub fn get_or_create_literal(&mut self, kind: Kind, text: String) -> &Grammar<Kind> {
    match self
      .t_cache
      .entry(kind.id())
      .or_insert(HashMap::new())
      .entry(Some(text.clone())) // TODO: prevent clone when unnecessary?
    {
      Entry::Occupied(o) => &self.map[&o.get()],
      Entry::Vacant(v) => {
        let id = self.map.len();
        v.insert(id);
        match self.map.entry(id) {
          // this should never happen
          Entry::Occupied(_) => panic!("Grammar with id {} already exists", id),
          Entry::Vacant(v) => v.insert(Grammar::new(GrammarType::T, kind, Some(text), id)),
        }
      }
    }
  }

  pub fn get_or_create_nt(&mut self, kind: Kind) -> &Grammar<Kind> {
    match self.nt_cache.entry(kind.id()) {
      Entry::Occupied(o) => &self.map[&o.get()],
      Entry::Vacant(v) => {
        let id = self.map.len();
        v.insert(id);
        match self.map.entry(id) {
          // this should never happen
          Entry::Occupied(_) => panic!("Grammar with id {} already exists", id),
          Entry::Vacant(v) => v.insert(Grammar::new(GrammarType::NT, kind, None, id)),
        }
      }
    }
  }
}
