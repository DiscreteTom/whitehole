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
  t_cache: HashMap<TokenKindId<TKind>, Rc<Grammar<TKind, NTKind>>>,
  /// This is used to check if a NT grammar is already created.
  nt_cache: HashMap<TokenKindId<NTKind>, Rc<Grammar<TKind, NTKind>>>,
  /// This is used to check if a literal grammar is already created.
  literal_cache: HashMap<String, Rc<Grammar<TKind, NTKind>>>,
  next_grammar_id: usize,
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> Default for GrammarRepo<TKind, NTKind> {
  fn default() -> Self {
    Self {
      t_cache: HashMap::new(),
      nt_cache: HashMap::new(),
      literal_cache: HashMap::new(),
      next_grammar_id: 0,
    }
  }
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> GrammarRepo<TKind, NTKind> {
  fn get_next_grammar_id(next_grammar_id: &mut usize) -> GrammarId {
    let id = GrammarId(*next_grammar_id);
    *next_grammar_id += 1;
    id
  }

  pub fn get_or_create_t(&mut self, kind: TKind) -> &Rc<Grammar<TKind, NTKind>> {
    match self.t_cache.entry(kind.id()) {
      // https://stackoverflow.com/questions/60129097/
      Entry::Occupied(o) => o.into_mut(),
      Entry::Vacant(v) => v.insert(Rc::new(Grammar::new(
        Self::get_next_grammar_id(&mut self.next_grammar_id),
        GrammarKind::T(kind),
      ))),
    }
  }

  pub fn get_or_create_literal(&mut self, text: String) -> &Rc<Grammar<TKind, NTKind>> {
    match self.literal_cache.entry(text.clone()) {
      // https://stackoverflow.com/questions/60129097/
      Entry::Occupied(o) => o.into_mut(),
      Entry::Vacant(v) => v.insert(Rc::new(Grammar::new(
        Self::get_next_grammar_id(&mut self.next_grammar_id),
        GrammarKind::Literal(text),
      ))),
    }
  }

  pub fn get_or_create_nt(&mut self, kind: NTKind) -> &Rc<Grammar<TKind, NTKind>> {
    match self.nt_cache.entry(kind.id()) {
      // https://stackoverflow.com/questions/60129097/
      Entry::Occupied(o) => o.into_mut(),
      Entry::Vacant(v) => v.insert(Rc::new(Grammar::new(
        Self::get_next_grammar_id(&mut self.next_grammar_id),
        GrammarKind::NT(kind),
      ))),
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
