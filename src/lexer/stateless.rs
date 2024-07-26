//! ## Design
//!
//! [`StatelessLexer`] doesn't hold lexer states or action states.
//! It is just a collection of immutable [`Action`]s, and it is immutable itself.
//! You can wrap it
//! with [`Rc`] to make it clone-able and re-use it across multiple lexers.
//!
//! The [`StatelessLexer`] implements all the core lexing features,
//! including expectation, fork, etc. If you
//! want a stateless experience, you can use the [`StatelessLexer`] directly,
//! but you may need to manage the lexer states and action states manually.
//!
//! ## The Lexing Process
//!
//! To optimize the runtime performance, the [`StatelessLexer`] will
//! pre-calculate and cache some action lists based on [`Action`]'s attributes
//! like [`Action::kind`] [`Action::head`], [`Action::literal`], etc.
//! When lexing, not all actions may be executed.
//! Here are the rules:
//!
//! ### Without Expectation
//!
//! If there is no expectation provided, the lexer will filter actions
//! by the first character of the rest of the input text, and actions' head matcher,
//! during the lexing loop.
//!
//! For example,
//! in one iteration of the lexing loop,
//! if the first character of the rest of the input text is `'a'`
//! only actions accepting `'a'` as the first character will be executed.
//!
//! ### With Expected Kind
//!
//! If there is an expected kind, the lexer will first ignore
//! non-muted actions with mismatched [`Action::kind`] before the lexing loop,
//! then ignore actions by the head matcher just like the case without expectation
//! during the lexing loop.
//!
//! ### With Expected Literal
//!
//! If there is an expected literal, the lexer will first ignore
//! non-muted actions with mismatched [`Action::literal`] before the lexing loop,
//! then check if the rest of the input text starts with the expected literal
//! during the lexing loop, if the literal doesn't match, all non-muted actions will be ignored,
//! finally ignore actions by the head matcher just like the case without expectation
//! during the lexing loop.
//!
//! ### With Both Expected Kind and Literal
//!
//! If there is both an expected kind and a literal, the lexer will first ignore
//! non-muted actions with mismatched [`Action::kind`] and mismatched [`Action::literal`]
//! before the lexing loop,
//! then check if the rest of the input text starts with the expected literal
//! during the lexing loop, if the literal doesn't match, all non-muted actions will be ignored,
//! finally ignore actions by the head matcher just like the case without expectation
//! during the lexing loop.

mod exec;
mod head_map;
mod lex;
mod literal_map;
mod options;
mod output;

pub use options::*;

use super::{action::Action, token::TokenKindId};
use head_map::HeadMap;
use literal_map::LiteralMap;
use std::{collections::HashMap, rc::Rc};

/// Stateless, immutable lexer.
pub struct StatelessLexer<Kind: 'static, ActionState = (), ErrorType = ()> {
  /// This is used to accelerate lexing by the first character when there is no expectation.
  head_map: HeadMap<Kind, ActionState, ErrorType>,
  /// This is used to accelerate expected lexing by the expected kind and the first character.
  kind_head_map: HashMap<TokenKindId<Kind>, HeadMap<Kind, ActionState, ErrorType>>,
  /// This is used to accelerate expected lexing by the expected literal.
  literal_map: LiteralMap<Kind, ActionState, ErrorType>,
  /// This is used to accelerate expected lexing by the expected kind and literal.
  kind_literal_map: HashMap<TokenKindId<Kind>, LiteralMap<Kind, ActionState, ErrorType>>,
  /// This is used to trim the lexer with muted actions.
  muted_head_map: HeadMap<Kind, ActionState, ErrorType>,
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
        .entry(a.kind().clone())
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
        kinds_action_map.get_mut(a.kind()).unwrap().push(a.clone());
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    // collect known chars/literals using all actions so we can re-use this map for all head/literal maps
    let known_char_map = HeadMap::collect_all_known(&actions);
    let known_literal_map = LiteralMap::collect_all_known(&actions);

    Self {
      kind_head_map: kinds_action_map
        .iter()
        .map(|(k, v)| (*k, HeadMap::new(v, known_char_map.clone())))
        .collect(),
      kind_literal_map: kinds_action_map
        .iter()
        .map(|(k, v)| {
          (
            *k,
            LiteralMap::new(v, known_literal_map.clone(), &known_char_map),
          )
        })
        .collect(),
      literal_map: LiteralMap::new(&actions, known_literal_map, &known_char_map),
      head_map: HeadMap::new(&actions, known_char_map.clone()),
      muted_head_map: HeadMap::new(
        &actions.into_iter().filter(|a| a.muted()).collect(),
        known_char_map,
      ),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::{exact, regex},
    token::{MockTokenKind, SubTokenKind, TokenKindIdBinding},
  };
  use whitehole_macros::_token_kind;

  #[_token_kind]
  #[derive(Debug, Clone)]
  pub enum MyKind {
    A,
    B,
  }

  fn r<S: 'static, E>(s: &str) -> Action<MockTokenKind<()>, S, E> {
    regex(s)
  }

  fn assert_actions_eq(
    actions: &Vec<Rc<Action<TokenKindIdBinding<MyKind>>>>,
    expected: Vec<Action<TokenKindIdBinding<MyKind>>>,
  ) {
    assert_eq!(actions.len(), expected.len());
    for i in 0..actions.len() {
      assert_eq!(actions[i].kind(), expected[i].kind());
      assert_eq!(actions[i].head(), expected[i].head());
      assert_eq!(actions[i].literal(), expected[i].literal());
      assert_eq!(actions[i].muted(), expected[i].muted());
    }
  }

  #[test]
  fn test_new_stateless_lexer() {
    let lexer: StatelessLexer<_> = StatelessLexer::new(
      vec![
        exact("a").bind(A),                              // A, "a", not muted
        exact("a").mute().bind(A),                       // A, "a", muted
        r("a").unchecked_head_in(['a']).bind(A),         // A, OneOf('a'), not muted
        r("a").unchecked_head_in(['a']).mute().bind(A),  // A, OneOf('a'), muted
        r("a").unchecked_head_not(['c']).bind(A),        // A, Not('c'), not muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").unchecked_head_unknown().bind(A),         // A, Unknown, not muted
        r("a").unchecked_head_unknown().mute().bind(A),  // A, Unknown, muted
        r("a").bind(A),                                  // A, no head, not muted
        r("a").mute().bind(A),                           // A, no head, muted
        exact("b").bind(B),                              // B, "b", not muted
        exact("b").mute().bind(B),                       // B, "b", muted
        r("b").unchecked_head_in(['b']).bind(B),         // B, OneOf('b'), not muted
        r("b").unchecked_head_in(['b']).mute().bind(B),  // B, OneOf('b'), muted
        r("b").unchecked_head_not(['c']).bind(B),        // B, Not('c'), not muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").unchecked_head_unknown().bind(B),         // B, Unknown, not muted
        r("b").unchecked_head_unknown().mute().bind(B),  // B, Unknown, muted
        r("b").bind(B),                                  // B, no head, not muted
        r("b").mute().bind(B),                           // B, no head, muted
      ]
      .into_iter()
      .map(Rc::new)
      .collect(),
    );

    // head_map
    assert_eq!(lexer.head_map.known_map().len(), ['a', 'b', 'c'].len());
    assert_actions_eq(
      &lexer.head_map.known_map()[&'a'],
      vec![
        exact("a").bind(A),                              // A, "a", not muted
        exact("a").mute().bind(A),                       // A, "a", muted
        r("a").unchecked_head_in(['a']).bind(A),         // A, OneOf('a'), not muted
        r("a").unchecked_head_in(['a']).mute().bind(A),  // A, OneOf('a'), muted
        r("a").unchecked_head_not(['c']).bind(A),        // A, Not('c'), not muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").bind(A),                                  // A, no head, not muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).bind(B),        // B, Not('c'), not muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").bind(B),                                  // B, no head, not muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    // `lexer.head_map.known_map()[&'b']` should be the similar to the above, skip.
    assert_actions_eq(
      &lexer.head_map.known_map()[&'c'],
      vec![
        r("a").bind(A),        // A, no head, not muted
        r("a").mute().bind(A), // A, no head, muted
        r("b").bind(B),        // B, no head, not muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_actions_eq(
      lexer.head_map.unknown_fallback(),
      vec![
        r("a").unchecked_head_not(['c']).bind(A), // A, Not('c'), not muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").unchecked_head_unknown().bind(A),  // A, Unknown, not muted
        r("a").unchecked_head_unknown().mute().bind(A), // A, Unknown, muted
        r("a").bind(A),                           // A, no head, not muted
        r("a").mute().bind(A),                    // A, no head, muted
        r("b").unchecked_head_not(['c']).bind(B), // B, Not('c'), not muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").unchecked_head_unknown().bind(B),  // B, Unknown, not muted
        r("b").unchecked_head_unknown().mute().bind(B), // B, Unknown, muted
        r("b").bind(B),                           // B, no head, not muted
        r("b").mute().bind(B),                    // B, no head, muted
      ],
    );

    // kind_head_map
    assert_eq!(lexer.kind_head_map.len(), ['A', 'B'].len());
    let kind_a_head_map = &lexer.kind_head_map[A::kind_id()];
    assert_eq!(kind_a_head_map.known_map().len(), ['a', 'b', 'c'].len());
    assert_actions_eq(
      &kind_a_head_map.known_map()[&'a'],
      vec![
        exact("a").bind(A),                              // A, "a", not muted
        exact("a").mute().bind(A),                       // A, "a", muted
        r("a").unchecked_head_in(['a']).bind(A),         // A, OneOf('a'), not muted
        r("a").unchecked_head_in(['a']).mute().bind(A),  // A, OneOf('a'), muted
        r("a").unchecked_head_not(['c']).bind(A),        // A, Not('c'), not muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").bind(A),                                  // A, no head, not muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_head_map.known_map()[&'b'],
      vec![
        r("a").unchecked_head_not(['c']).bind(A), // A, Not('c'), not muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").bind(A),                           // A, no head, not muted
        r("a").mute().bind(A),                    // A, no head, muted
        exact("b").mute().bind(B),                // B, "b", muted
        r("b").unchecked_head_in(['b']).mute().bind(B), // B, OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                    // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_head_map.known_map()[&'c'],
      vec![
        r("a").bind(A),        // A, no head, not muted
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_head_map.unknown_fallback(),
      vec![
        r("a").unchecked_head_not(['c']).bind(A), // A, Not('c'), not muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").unchecked_head_unknown().bind(A),  // A, Unknown, not muted
        r("a").unchecked_head_unknown().mute().bind(A), // A, Unknown, muted
        r("a").bind(A),                           // A, no head, not muted
        r("a").mute().bind(A),                    // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").unchecked_head_unknown().mute().bind(B), // B, Unknown, muted
        r("b").mute().bind(B),                    // B, no head, muted
      ],
    );
    // `lexer.kind_head_map[B::kind_id()]` should be the similar to the above, skip.

    // literal_map
    assert_eq!(lexer.literal_map.known_map().len(), ["a", "b"].len());
    let literal_a_head_map = &lexer.literal_map.known_map().get("a").unwrap().head_map;
    assert_eq!(literal_a_head_map.known_map().len(), ['a', 'b', 'c'].len());
    assert_actions_eq(
      literal_a_head_map.known_map().get(&'a').unwrap(),
      vec![
        exact("a").bind(A),                              // A, "a", not muted
        exact("a").mute().bind(A),                       // A, "a", muted
        r("a").unchecked_head_in(['a']).mute().bind(A),  // A, OneOf('a'), muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_actions_eq(
      literal_a_head_map.known_map().get(&'b').unwrap(),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        exact("b").mute().bind(B),                       // B, "b", muted
        r("b").unchecked_head_in(['b']).mute().bind(B),  // B, OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_actions_eq(
      literal_a_head_map.known_map().get(&'c').unwrap(),
      vec![
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_actions_eq(
      literal_a_head_map.unknown_fallback(),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").unchecked_head_unknown().mute().bind(A),  // A, Unknown, muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").unchecked_head_unknown().mute().bind(B),  // B, Unknown, muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    // literal_b_head_map should be similar to literal_a_head_map, skip
    let literal_a_muted_head_map = &lexer
      .literal_map
      .known_map()
      .get("a")
      .unwrap()
      .muted_head_map;
    assert_eq!(
      literal_a_muted_head_map.known_map().len(),
      ['a', 'b', 'c'].len()
    );
    assert_actions_eq(
      literal_a_muted_head_map.known_map().get(&'a').unwrap(),
      vec![
        exact("a").mute().bind(A),                       // A, "a", muted
        r("a").unchecked_head_in(['a']).mute().bind(A),  // A, OneOf('a'), muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_actions_eq(
      literal_a_muted_head_map.known_map().get(&'b').unwrap(),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        exact("b").mute().bind(B),                       // B, "b", muted
        r("b").unchecked_head_in(['b']).mute().bind(B),  // B, OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_actions_eq(
      literal_a_muted_head_map.known_map().get(&'c').unwrap(),
      vec![
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_actions_eq(
      literal_a_muted_head_map.unknown_fallback(),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").unchecked_head_unknown().mute().bind(A),  // A, Unknown, muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").unchecked_head_unknown().mute().bind(B),  // B, Unknown, muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    // literal_b_muted_head_map should be similar to literal_a_muted_head_map, skip

    // kind_literal_map
    assert_eq!(lexer.kind_literal_map.len(), ["A", "B"].len());
    let kind_a_literal_map = &lexer.kind_literal_map[A::kind_id()];
    assert_eq!(kind_a_literal_map.known_map().len(), ["a", "b"].len());
    assert_actions_eq(
      &kind_a_literal_map.known_map()["a"].head_map.known_map()[&'a'],
      vec![
        exact("a").bind(A),                              // A, "a", not muted
        exact("a").mute().bind(A),                       // A, "a", muted
        r("a").unchecked_head_in(['a']).mute().bind(A),  // A, OneOf('a'), muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_literal_map.known_map()["a"].head_map.known_map()[&'b'],
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        exact("b").mute().bind(B),                       // B, "b", muted
        r("b").unchecked_head_in(['b']).mute().bind(B),  // B, OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_literal_map.known_map()["a"].head_map.known_map()[&'c'],
      vec![
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_literal_map.known_map()["a"]
        .head_map
        .unknown_fallback(),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").unchecked_head_unknown().mute().bind(A),  // A, Unknown, muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").unchecked_head_unknown().mute().bind(B),  // B, Unknown, muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_literal_map.known_map()["a"]
        .muted_head_map
        .known_map()[&'a'],
      vec![
        exact("a").mute().bind(A),                       // A, "a", muted
        r("a").unchecked_head_in(['a']).mute().bind(A),  // A, OneOf('a'), muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_literal_map.known_map()["a"]
        .muted_head_map
        .known_map()[&'b'],
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        exact("b").mute().bind(B),                       // B, "b", muted
        r("b").unchecked_head_in(['b']).mute().bind(B),  // B, OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_literal_map.known_map()["a"]
        .muted_head_map
        .known_map()[&'c'],
      vec![
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_literal_map.known_map()["a"]
        .muted_head_map
        .unknown_fallback(),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").unchecked_head_unknown().mute().bind(A),  // A, Unknown, muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").unchecked_head_unknown().mute().bind(B),  // B, Unknown, muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    // kind_a_literal_map.known_map()["b"] should be similar to kind_a_literal_map.known_map()["a"], skip
    // kind_b_literal_map should be similar to kind_a_literal_map, skip
  }
}
