use crate::{
  lexer::token::{TokenKind, TokenKindId},
  parser::elr::grammar::grammar::{Grammar, GrammarId, GrammarKind},
};
use std::collections::{hash_map::Entry, HashMap};

pub struct GrammarRepo<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> {
  /// This is used to check if a T grammar is already created.
  t_cache: HashMap<TokenKindId<TKind>, GrammarId>,
  /// This is used to check if a NT grammar is already created.
  nt_cache: HashMap<TokenKindId<NTKind>, GrammarId>,
  /// This is used to check if a literal grammar is already created.
  literal_cache: HashMap<String, GrammarId>,
  /// This is used to get the grammar by id.
  // TODO: is this needed? can we just store grammar in caches?
  map: HashMap<GrammarId, Grammar<TKind, NTKind>>,
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> Default for GrammarRepo<TKind, NTKind> {
  fn default() -> Self {
    Self {
      t_cache: HashMap::new(),
      nt_cache: HashMap::new(),
      literal_cache: HashMap::new(),
      map: HashMap::new(),
    }
  }
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> GrammarRepo<TKind, NTKind> {
  pub fn get_or_create_t(&mut self, kind: TKind) -> &Grammar<TKind, NTKind> {
    match self.t_cache.entry(kind.id()) {
      Entry::Occupied(o) => &self.map[&o.get()],
      Entry::Vacant(v) => {
        let id = GrammarId(self.map.len());
        v.insert(id);
        match self.map.entry(id) {
          // this should never happen
          Entry::Occupied(_) => panic!("Grammar with id {:?} already exists", id),
          Entry::Vacant(v) => v.insert(Grammar::new(id, GrammarKind::T(kind))),
        }
      }
    }
  }

  pub fn get_or_create_literal(&mut self, text: String) -> &Grammar<TKind, NTKind> {
    match self.literal_cache.entry(text.clone()) {
      Entry::Occupied(o) => &self.map[&o.get()],
      Entry::Vacant(v) => {
        let id = GrammarId(self.map.len());
        v.insert(id);
        match self.map.entry(id) {
          // this should never happen
          Entry::Occupied(_) => panic!("Grammar with id {:?} already exists", id),
          Entry::Vacant(v) => v.insert(Grammar::new(id, GrammarKind::Literal(text))),
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
          Entry::Vacant(v) => v.insert(Grammar::new(id, GrammarKind::NT(kind))),
        }
      }
    }
  }
}
