mod common;
pub mod lex;
pub mod trim;

use super::{
  action::{Action, ActionInputRestHeadMatcher},
  token::{TokenKind, TokenKindId},
  Lexer,
};
use std::{collections::HashMap, rc::Rc};

/// Stateless, immutable lexer.
pub struct StatelessLexer<Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  /// All actions.
  actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  /// This is used to accelerate expected lexing.
  kind_map: HashMap<TokenKindId<Kind>, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
  /// This is used to accelerate lexing by the first character.
  head_map: HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
  /// This is used to accelerate trimming.
  maybe_muted_actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
}

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  pub fn new(actions: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    // TODO: move the build process into builder.generate?
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

    let mut head_map = HashMap::new();
    // collect all known chars
    for a in &actions {
      if let Some(head_matcher) = a.head_matcher() {
        for c in match head_matcher {
          ActionInputRestHeadMatcher::OneOf(set) => set,
          ActionInputRestHeadMatcher::Not(set) => set,
        } {
          head_map.entry(*c).or_insert(Vec::new());
        }
      }
    }
    // fill the head_map
    for a in &actions {
      if let Some(head_matcher) = a.head_matcher() {
        match head_matcher {
          ActionInputRestHeadMatcher::OneOf(set) => {
            for c in set {
              head_map.get_mut(c).unwrap().push(a.clone());
            }
          }
          ActionInputRestHeadMatcher::Not(set) => {
            for (c, vec) in head_map.iter_mut() {
              if !set.contains(c) {
                vec.push(a.clone());
              }
            }
          }
        }
      } else {
        // no head matcher, add to all chars
        for vec in head_map.values_mut() {
          vec.push(a.clone());
        }
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    StatelessLexer {
      kind_map: action_map,
      maybe_muted_actions: actions
        .iter()
        .filter(|a| a.maybe_muted)
        .map(|a| a.clone())
        .collect(),
      head_map,
      actions,
    }
  }

  pub fn actions(&self) -> &[Rc<Action<Kind, ActionState, ErrorType>>] {
    &self.actions
  }
  pub fn kind_map(
    &self,
  ) -> &HashMap<TokenKindId<Kind>, Vec<Rc<Action<Kind, ActionState, ErrorType>>>> {
    &self.kind_map
  }
  pub fn head_map(&self) -> &HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>> {
    &self.head_map
  }
  pub fn maybe_muted_actions(&self) -> &[Rc<Action<Kind, ActionState, ErrorType>>] {
    &self.maybe_muted_actions
  }

  /// Consume self, create a new lexer with the provided buffer.
  pub fn into_lexer(self, buffer: &str) -> Lexer<Kind, ActionState, ErrorType> {
    Lexer::new(Rc::new(self), buffer)
  }
}
