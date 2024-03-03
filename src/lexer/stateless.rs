mod common;
mod lex;
mod trim;

pub use lex::{StatelessLexOptions, StatelessLexOutput};

use super::{
  action::{Action, ActionInputRestHeadMatcher},
  token::{TokenKind, TokenKindId},
  Lexer,
};
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

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
{
  pub fn new(actions: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    // TODO: move the build process into builder.generate?
    let actions = actions.into_iter().map(Rc::new).collect::<Vec<_>>();

    let mut kind_map = HashMap::new();
    // prepare kind map, add value for all possible kinds
    for a in &actions {
      for k in a.possible_kinds() {
        kind_map.entry(k.clone()).or_insert(Vec::new());
      }
    }
    // fill kind_map
    for a in &actions {
      if a.maybe_muted {
        // maybe muted, add to all kinds
        for (_, vec) in kind_map.iter_mut() {
          vec.push(a.clone());
        }
      } else {
        // never muted, only add to possible kinds
        for k in a.possible_kinds() {
          kind_map.get_mut(k).unwrap().push(a.clone());
        }
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    let maybe_muted_actions = actions
      .iter()
      .filter(|a| a.maybe_muted)
      .map(|a| a.clone())
      .collect();

    StatelessLexer {
      kind_head_map: kind_map
        .iter()
        .map(|(k, v)| (k.clone(), Self::calc_head_map(&v)))
        .collect(),
      head_map: Self::calc_head_map(&actions),
      maybe_muted_head_map: Self::calc_head_map(&maybe_muted_actions),
      actions,
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

  /// Consume self, create a new lexer with the provided text.
  pub fn into_lexer(self, text: &str) -> Lexer<Kind, ActionState, ErrorType>
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    ActionState: Default, // TODO: add a function that accept an action state instead of default
  {
    Lexer::new(Rc::new(self), text)
  }

  fn calc_head_map(
    actions: &Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  ) -> ActionHeadMap<Kind, ActionState, ErrorType> {
    let mut head_map = ActionHeadMap {
      known_map: HashMap::new(),
      unknown_fallback: Vec::new(),
    };
    // collect all known chars
    for a in actions {
      if let Some(head_matcher) = a.head_matcher() {
        for c in match head_matcher {
          ActionInputRestHeadMatcher::OneOf(set) => set,
          ActionInputRestHeadMatcher::Not(set) => set,
          ActionInputRestHeadMatcher::Unknown => continue,
        } {
          head_map.known_map.entry(*c).or_insert(Vec::new());
        }
      }
    }
    // fill the head_map
    for a in actions {
      if let Some(head_matcher) = a.head_matcher() {
        match head_matcher {
          ActionInputRestHeadMatcher::OneOf(set) => {
            for c in set {
              head_map.known_map.get_mut(c).unwrap().push(a.clone());
            }
          }
          ActionInputRestHeadMatcher::Not(set) => {
            for (c, vec) in head_map.known_map.iter_mut() {
              if !set.contains(c) {
                vec.push(a.clone());
              }
            }
            head_map.unknown_fallback.push(a.clone());
          }
          ActionInputRestHeadMatcher::Unknown => {
            head_map.unknown_fallback.push(a.clone());
          }
        }
      } else {
        // no head matcher, add to all known chars
        for vec in head_map.known_map.values_mut() {
          vec.push(a.clone());
        }
        // and unknown fallback
        head_map.unknown_fallback.push(a.clone());
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    head_map
  }
}
