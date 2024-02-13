use crate::{
  lexer::token::{TokenKind, TokenKindId},
  parser::elr::grammar::grammar::{Grammar, GrammarId, GrammarKind},
};
use std::{
  collections::{hash_map::Entry, HashMap},
  rc::Rc,
};

pub struct GrammarRepo<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> {
  /// This is used to check if a T grammar is already created.
  t_cache: HashMap<TokenKindId<TKind>, GrammarId>,
  /// This is used to check if a NT grammar is already created.
  nt_cache: HashMap<TokenKindId<NTKind>, GrammarId>,
  /// This is used to check if a literal grammar is already created.
  literal_cache: HashMap<String, GrammarId>,
  /// This is used to get the grammar by id.
  // TODO: is this needed? can we just store grammar in caches?
  grammars: Vec<Rc<Grammar<TKind, NTKind>>>,
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> Default for GrammarRepo<TKind, NTKind> {
  fn default() -> Self {
    Self {
      t_cache: HashMap::new(),
      nt_cache: HashMap::new(),
      literal_cache: HashMap::new(),
      grammars: Vec::new(),
    }
  }
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> GrammarRepo<TKind, NTKind> {
  pub fn get_or_create_t(&mut self, kind: TKind) -> &Rc<Grammar<TKind, NTKind>> {
    match self.t_cache.entry(kind.id()) {
      Entry::Occupied(o) => &self.grammars[o.get().0],
      Entry::Vacant(v) => {
        let id = GrammarId(self.grammars.len());
        v.insert(id);
        self
          .grammars
          .push(Rc::new(Grammar::new(id, GrammarKind::T(kind))));
        &self.grammars[id.0]
      }
    }
  }

  pub fn get_or_create_literal(&mut self, text: String) -> &Rc<Grammar<TKind, NTKind>> {
    match self.literal_cache.entry(text.clone()) {
      Entry::Occupied(o) => &self.grammars[o.get().0],
      Entry::Vacant(v) => {
        let id = GrammarId(self.grammars.len());
        v.insert(id);
        self
          .grammars
          .push(Rc::new(Grammar::new(id, GrammarKind::Literal(text))));
        &self.grammars[id.0]
      }
    }
  }

  pub fn get_or_create_nt(&mut self, kind: NTKind) -> &Rc<Grammar<TKind, NTKind>> {
    match self.nt_cache.entry(kind.id()) {
      Entry::Occupied(o) => &self.grammars[o.get().0],
      Entry::Vacant(v) => {
        let id = GrammarId(self.grammars.len());
        v.insert(id);
        self
          .grammars
          .push(Rc::new(Grammar::new(id, GrammarKind::NT(kind))));
        &self.grammars[id.0]
      }
    }
  }

  pub fn get_or_create(&mut self, kind: GrammarKind<TKind, NTKind>) -> &Rc<Grammar<TKind, NTKind>> {
    match kind {
      GrammarKind::T(kind) => self.get_or_create_t(kind),
      GrammarKind::NT(kind) => self.get_or_create_nt(kind),
      GrammarKind::Literal(text) => self.get_or_create_literal(text),
    }
  }
}
