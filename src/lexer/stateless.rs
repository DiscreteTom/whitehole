mod common;
pub mod lex;
pub mod trim;

use super::{
  action::Action,
  token::{TokenKind, TokenKindId},
  Lexer,
};
use std::{collections::HashMap, rc::Rc};

/// Stateless, immutable lexer.
pub struct StatelessLexer<Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  /// All actions.
  actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  /// This is used to accelerate expected lexing.
  action_map: HashMap<TokenKindId, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>, // TODO: don't overuse Rc, can we just use a reference?
  /// This is used to accelerate trimming.
  maybe_muted_actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
}

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn new(actions: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    let actions = actions.into_iter().map(Rc::new).collect::<Vec<_>>();

    let mut action_map = HashMap::new();
    // prepare action map, add vec for all possible kinds
    for a in &actions {
      for k in a.possible_kinds() {
        action_map.entry(k.clone()).or_insert(Vec::new());
      }
    }
    // fill action_map
    for a in &actions {
      if a.maybe_muted {
        // maybe muted, add to all kinds
        for (_, vec) in action_map.iter_mut() {
          vec.push(a.clone());
        }
      } else {
        // never muted, only add to possible kinds
        for k in a.possible_kinds() {
          action_map.get_mut(k).unwrap().push(a.clone());
        }
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    StatelessLexer {
      action_map,
      maybe_muted_actions: actions
        .iter()
        .filter(|a| a.maybe_muted)
        .map(|a| a.clone())
        .collect(),
      actions,
    }
  }

  pub fn actions(&self) -> &[Rc<Action<Kind, ActionState, ErrorType>>] {
    &self.actions
  }
  pub fn action_map(&self) -> &HashMap<TokenKindId, Vec<Rc<Action<Kind, ActionState, ErrorType>>>> {
    &self.action_map
  }
  pub fn maybe_muted_actions(&self) -> &[Rc<Action<Kind, ActionState, ErrorType>>] {
    &self.maybe_muted_actions
  }

  /// Consume self, create a new lexer with the provided buffer.
  pub fn into_lexer(self, buffer: &str) -> Lexer<Kind, ActionState, ErrorType> {
    Lexer::new(Rc::new(self), buffer)
  }
}
