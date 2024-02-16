use super::grammar::Grammar;
use crate::lexer::token::{TokenKind, TokenKindId};
use std::{collections::HashMap, rc::Rc};

pub struct GrammarMap<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> {
  token_kind_grammar_map: HashMap<TokenKindId<TKind>, Rc<Grammar<TKind, NTKind>>>,
  literal_grammar_map: HashMap<String, Rc<Grammar<TKind, NTKind>>>,
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> GrammarMap<TKind, NTKind> {
  pub fn new(
    token_kind_grammar_map: HashMap<TokenKindId<TKind>, Rc<Grammar<TKind, NTKind>>>,
    literal_grammar_map: HashMap<String, Rc<Grammar<TKind, NTKind>>>,
  ) -> Self {
    Self {
      token_kind_grammar_map,
      literal_grammar_map,
    }
  }

  pub fn token_kind_grammar_map(&self) -> &HashMap<TokenKindId<TKind>, Rc<Grammar<TKind, NTKind>>> {
    &self.token_kind_grammar_map
  }
  pub fn literal_grammar_map(&self) -> &HashMap<String, Rc<Grammar<TKind, NTKind>>> {
    &self.literal_grammar_map
  }
}
