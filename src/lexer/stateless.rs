mod common;
mod lex;
mod options;

pub use lex::*;
pub use options::*;

use super::{action::Action, token::TokenKindId};
use std::{collections::HashMap, rc::Rc};

pub struct ActionHeadMap<Kind, ActionState, ErrorType> {
  /// Store actions for known chars.
  pub known_map: HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
  /// Store actions for unknown chars.
  pub unknown_fallback: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
}

/// Stateless, immutable lexer.
pub struct StatelessLexer<Kind, ActionState, ErrorType> {
  /// All actions.
  actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  /// This is used to accelerate lexing by the first character when no expected kind.
  head_map: ActionHeadMap<Kind, ActionState, ErrorType>,
  /// This is used to accelerate expected lexing by the expected kind and the first character.
  kind_head_map: HashMap<TokenKindId<Kind>, ActionHeadMap<Kind, ActionState, ErrorType>>,
  /// This is used to accelerate trimming by the first character.
  maybe_muted_head_map: ActionHeadMap<Kind, ActionState, ErrorType>,
}

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  pub fn new(
    actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
    head_map: ActionHeadMap<Kind, ActionState, ErrorType>,
    kind_head_map: HashMap<TokenKindId<Kind>, ActionHeadMap<Kind, ActionState, ErrorType>>,
    maybe_muted_head_map: ActionHeadMap<Kind, ActionState, ErrorType>,
  ) -> Self {
    StatelessLexer {
      actions,
      head_map,
      kind_head_map,
      maybe_muted_head_map,
    }
  }

  pub fn actions(&self) -> &[Rc<Action<Kind, ActionState, ErrorType>>] {
    &self.actions
  }
  pub fn head_map(&self) -> &ActionHeadMap<Kind, ActionState, ErrorType> {
    &self.head_map
  }
  pub fn kind_head_map(
    &self,
  ) -> &HashMap<TokenKindId<Kind>, ActionHeadMap<Kind, ActionState, ErrorType>> {
    &self.kind_head_map
  }
  pub fn maybe_muted_head_map(&self) -> &ActionHeadMap<Kind, ActionState, ErrorType> {
    &self.maybe_muted_head_map
  }
}
