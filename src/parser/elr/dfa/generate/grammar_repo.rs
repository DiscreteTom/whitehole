use crate::{
  lexer::token::{TokenKind, TokenKindId},
  parser::elr::grammar::grammar::{Grammar, GrammarId, GrammarKind},
};
use std::collections::{hash_map::Entry, HashMap};

pub struct GrammarRepo<TKind: TokenKind, NTKind: TokenKind> {
  /// This is used to check if a T grammar is already created.
  t_cache: HashMap<TokenKindId, HashMap<Option<String>, GrammarId>>,
  /// This is used to check if a NT grammar is already created.
  nt_cache: HashMap<TokenKindId, GrammarId>,
  /// This is used to get the grammar by id.
  // TODO: is this needed? can we just store grammar in caches?
  map: HashMap<GrammarId, Grammar<TKind, NTKind>>,
}

impl<TKind: TokenKind, NTKind: TokenKind> Default for GrammarRepo<TKind, NTKind> {
  fn default() -> Self {
    Self {
      t_cache: HashMap::new(),
      nt_cache: HashMap::new(),
      map: HashMap::new(),
    }
  }
}

impl<TKind: TokenKind, NTKind: TokenKind> GrammarRepo<TKind, NTKind> {
  pub fn get_or_create_t(&mut self, kind: TKind) -> &Grammar<TKind, NTKind> {
    match self
      .t_cache
      .entry(kind.id())
      .or_insert(HashMap::new())
      .entry(None) // None for no text field
    {
      Entry::Occupied(o) => &self.map[&o.get()],
      Entry::Vacant(v) => {
        let id = GrammarId(self.map.len());
        v.insert(id);
        match self.map.entry(id) {
          // this should never happen
          Entry::Occupied(_) => panic!("Grammar with id {:?} already exists", id),
          Entry::Vacant(v) => v.insert(Grammar::new(id,GrammarKind::T(kind) , None)),
        }
      }
    }
  }

  pub fn get_or_create_literal(&mut self, kind: TKind, text: String) -> &Grammar<TKind, NTKind> {
    match self
      .t_cache
      .entry(kind.id())
      .or_insert(HashMap::new())
      .entry(Some(text.clone())) // TODO: prevent clone when unnecessary?
    {
      Entry::Occupied(o) => &self.map[&o.get()],
      Entry::Vacant(v) => {
        let id = GrammarId(self.map.len());
        v.insert(id);
        match self.map.entry(id) {
          // this should never happen
          Entry::Occupied(_) => panic!("Grammar with id {:?} already exists", id),
          Entry::Vacant(v) => v.insert(Grammar::new(id,GrammarKind::T(kind), Some(text))),
        }
      }
    }
  }

  pub fn get_or_create_nt(&mut self, kind: NTKind) -> &Grammar<TKind, NTKind> {
    match self.nt_cache.entry(kind.id()) {
      Entry::Occupied(o) => &self.map[&o.get()],
      Entry::Vacant(v) => {
        let id = GrammarId(self.map.len());
        v.insert(id);
        match self.map.entry(id) {
          // this should never happen
          Entry::Occupied(_) => panic!("Grammar with id {:?} already exists", id),
          Entry::Vacant(v) => v.insert(Grammar::new(id, GrammarKind::NT(kind), None)),
        }
      }
    }
  }
}
