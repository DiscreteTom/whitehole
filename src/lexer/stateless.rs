mod exec;
mod head_map;
mod lex;
mod options;

pub use options::*;

use super::{action::Action, token::TokenKindId};
use head_map::HeadMap;
use std::{collections::HashMap, rc::Rc};

/// Stateless, immutable lexer.
pub struct StatelessLexer<Kind: 'static, ActionState, ErrorType> {
  /// All actions.
  actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  /// This is used to accelerate lexing by the first character when no expected kind.
  head_map: HeadMap<Kind, ActionState, ErrorType>,
  /// This is used to accelerate expected lexing by the expected kind and the first character.
  kind_head_map: HashMap<TokenKindId<Kind>, HeadMap<Kind, ActionState, ErrorType>>,
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

    let kind_head_map = kinds_action_map
      .iter()
      .map(|(k, v)| (k.clone(), HeadMap::new(&v)))
      .collect();
    let head_map = HeadMap::new(&actions);

    Self {
      actions,
      head_map,
      kind_head_map,
    }
  }

  pub fn actions(&self) -> &[Rc<Action<Kind, ActionState, ErrorType>>] {
    &self.actions
  }
}
