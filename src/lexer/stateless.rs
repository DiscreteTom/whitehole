mod common;
mod head_map;
mod lex;
mod options;

pub use head_map::*;
pub use lex::*;
pub use options::*;

use super::{action::Action, token::TokenKindId};
use std::{collections::HashMap, rc::Rc};

/// Stateless, immutable lexer.
pub struct StatelessLexer<Kind, ActionState, ErrorType> {
  /// All actions.
  actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  /// This is used to accelerate lexing by the first character when no expected kind.
  head_map: ActionHeadMap<Kind, ActionState, ErrorType>,
  /// This is used to accelerate expected lexing by the expected kind and the first character.
  kind_head_map: HashMap<TokenKindId<Kind>, ActionHeadMap<Kind, ActionState, ErrorType>>,
  /// This is used to accelerate trimming by the first character.
  maybe_muted_head_map: ActionHeadMap<Kind, ActionState, ErrorType>, // TODO: remove this?
}

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  pub fn new(actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>) -> Self {
    // known kinds => actions
    let mut kinds_action_map = HashMap::new();
    // prepare kind map, add value for all known possible kinds
    // this has to be done before filling the map
    // because we need to iter over all possible kinds when filling the map
    for a in &actions {
      kinds_action_map
        .entry(a.kind_id().clone())
        .or_insert(Vec::new());
    }
    // fill it
    for a in &actions {
      if a.maybe_muted() {
        // maybe muted, add to all kinds
        for (_, vec) in kinds_action_map.iter_mut() {
          vec.push(a.clone());
        }
      } else {
        // never muted, only add to possible kinds
        kinds_action_map
          .get_mut(a.kind_id())
          .unwrap()
          .push(a.clone());
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    let maybe_muted_actions = actions
      .iter()
      .filter(|a| a.maybe_muted())
      .map(|a| a.clone())
      .collect();

    let kind_head_map = kinds_action_map
      .iter()
      .map(|(k, v)| (k.clone(), ActionHeadMap::new(&v)))
      .collect();
    let head_map = ActionHeadMap::new(&actions);
    let maybe_muted_head_map = ActionHeadMap::new(&maybe_muted_actions);

    Self {
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
