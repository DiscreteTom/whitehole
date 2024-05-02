//! ## Design
//!
//! [`StatelessLexer`] doesn't hold lexer states or action states.
//! It is just a collection of immutable [`Action`]s, and it is immutable itself.
//! We can wrap it
//! with [`Rc`] to make it clone-able and re-use it across multiple lexers.
//!
//! The [`StatelessLexer`] implements all the core lexing features,
//! including expectation, fork, etc. If we
//! want a stateless experience, we can use the [`StatelessLexer`] directly,
//! but we may need to manage the lexer states and action states manually.
//!
//! ## Lexing Process
//!
//! To optimize the runtime performance, the [`StatelessLexer`] will
//! pre-calculate and cache some action lists based on [`Action`]'s attributes
//! like [`Action::kind_id`] [`Action::head_matcher`], [`Action::literal`], etc.
//! When lexing, maybe not all of the actions will be evaluated/executed.
//! Here are the rules:
//!
//! ### Without Expectation
//!
//! If there is no expectation provided, the lexer will filter actions
//! by the first character of the rest of input text, and action's head matcher.
//!
//! For example, if the first character of the rest of input text is `'a'`,
//! only actions accepting `'a'` as the first character will be evaluated.
//!
//! ### With Expected Kind
//!
//! If there is an expected kind, the lexer will first ignore actions
//! with different [`Action::kind_id`] (unless muted), then ignore actions by the head matcher
//! just like the case without expectation.
//!
//! ### With Expected Literal
//!
//! If there is an expected literal, the lexer will ignore actions
//! with no or mismatched [`Action::literal`] (unless muted).
//! We don't need to check the head matcher in this case.
//!
//! ### With Both Expected Kind and Literal
//!
//! If there is both an expected kind and a literal, the lexer will first ignore actions
//! with different [`Action::kind_id`] (unless muted), then ignore actions
//! with no or mismatched [`Action::literal`] (unless muted).
//! We don't need to check the head matcher in this case.

mod exec;
mod head_map;
mod lex;
mod literal_map;
mod options;

pub use options::*;

use super::{action::Action, token::TokenKindId};
use head_map::HeadMap;
use literal_map::LiteralMap;
use std::{collections::HashMap, rc::Rc};

/// Stateless, immutable lexer.
pub struct StatelessLexer<Kind: 'static, ActionState, ErrorType> {
  /// All actions.
  actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  /// This is used to accelerate lexing by the first character when there is no expectation.
  head_map: HeadMap<Kind, ActionState, ErrorType>,
  /// This is used to accelerate expected lexing by the expected kind and the first character.
  kind_head_map: HashMap<TokenKindId<Kind>, HeadMap<Kind, ActionState, ErrorType>>,
  /// This is used to accelerate expected lexing by the expected literal.
  literal_map: LiteralMap<Kind, ActionState, ErrorType>,
  /// This is used to accelerate expected lexing by the expected kind and literal.
  kind_literal_map: HashMap<TokenKindId<Kind>, LiteralMap<Kind, ActionState, ErrorType>>,
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
      if a.muted() {
        // muted, add to all kinds
        for (_, vec) in kinds_action_map.iter_mut() {
          vec.push(a.clone());
        }
      } else {
        // non-muted, only add to possible kinds
        kinds_action_map
          .get_mut(a.kind_id())
          .unwrap()
          .push(a.clone());
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    Self {
      head_map: HeadMap::new(&actions),
      kind_head_map: kinds_action_map
        .iter()
        .map(|(k, v)| (*k, HeadMap::new(v)))
        .collect(),
      literal_map: LiteralMap::new(&actions),
      kind_literal_map: kinds_action_map
        .iter()
        .map(|(k, v)| (*k, LiteralMap::new(v)))
        .collect(),
      actions,
    }
  }

  pub fn actions(&self) -> &[Rc<Action<Kind, ActionState, ErrorType>>] {
    &self.actions
  }
}
