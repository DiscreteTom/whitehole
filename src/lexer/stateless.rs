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
//! Macroscopically, the lexing process will execute your actions one by one,
//! in the order you provided, until a token is emitted or the input text is exhausted.
//!
//! However, to optimize the runtime performance, the [`StatelessLexer`] will
//! skip some actions based on [`Action`]'s attributes
//! like [`Action::kind`] [`Action::head`], [`Action::literal`], etc.
//! So when lexing, not all actions may be executed, but the order of actions
//! will be preserved.
//! Here are the skipping rules:
//!
//! ### Without Expectation
//!
//! If there is no expectation provided, the lexer will skip actions
//! by the first character of the rest of the input text, and actions' head matcher ([`Action::head`]),
//! during the lexing loop.
//!
//! For example,
//! in one iteration of the lexing loop,
//! if the first character of the rest of the input text is `'a'`
//! only actions accepting `'a'` as the first character will be executed.
//!
//! If an action has no head matcher, it will be executed no matter what the first character is.
//! So it is recommended to add a head matcher to all actions to make the lexer faster.
//!
//! ### With Expected Kind
//!
//! If there is an expected kind, the lexer will first skip
//! non-muted actions with mismatched [`Action::kind`]
//! (muted actions won't emit tokens so even they have mismatched kind they will be executed),
//! then skip actions by the head matcher just like the case without expectation
//! during the lexing loop.
//!
//! ### With Expected Literal
//!
//! If there is an expected literal, the lexer will first skip
//! non-muted actions with mismatched [`Action::literal`]
//! (muted actions won't emit tokens so even they have mismatched literal they will be executed),
//! then check if the rest of the input text starts with the expected literal
//! during the lexing loop. If the literal doesn't match, all non-muted actions will be skipped,
//! only muted actions will be executed.
//! Finally skip actions by the head matcher just like the case without expectation
//! during the lexing loop.
//!
//! ### With Both Expected Kind and Literal
//!
//! If there is both an expected kind and a literal, the lexer will first skip
//! non-muted actions with mismatched [`Action::kind`] and mismatched [`Action::literal`]
//! before the lexing loop,
//! then check if the rest of the input text starts with the expected literal
//! during the lexing loop. If the literal doesn't match, all non-muted actions will be skipped,
//! only muted actions will be executed.
//! Finally skip actions by the head matcher just like the case without expectation
//! during the lexing loop.
//!
//! ### Trim
//!
//! ## Caveats
//!
//! Be careful with stateless lexer.
//!
//! ## For Developers
//!
//! Here is the recommended order of reading the source code:
//!
//! - [`self::head_map`]
//! - [`self::literal_map`]
//! - [`self`]
//! - [`self::options`]
//! - [`self::lex`]
//! - [`self::trim`]
//! - [`self::utils`]

mod head_map;
mod lex;
mod literal_map;
mod options;
mod trim;
mod utils;

pub use options::*;

use super::{
  action::{Action, GeneralAction},
  token::TokenKindId,
};
use head_map::HeadMap;
use literal_map::LiteralMap;
use std::collections::HashMap;

/// Stateless, immutable lexer.
pub struct StatelessLexer<Kind: 'static, ActionState = (), ErrorType = ()> {
  /// This is used to accelerate lexing by actions' head matcher when there is no expectation.
  /// This is pre-calculated to optimize the runtime performance.
  head_map: HeadMap<Kind, ActionState, ErrorType>,
  /// This is used to accelerate expected lexing by the expected kind and actions' head matcher.
  /// This is pre-calculated to optimize the runtime performance.
  kind_head_map: HashMap<&'static TokenKindId<Kind>, HeadMap<Kind, ActionState, ErrorType>>,
  /// This is used to accelerate expected lexing by the expected literal and actions' head matcher.
  /// This is pre-calculated to optimize the runtime performance.
  literal_map: LiteralMap<Kind, ActionState, ErrorType>,
  /// This is used to accelerate expected lexing by the expected kind, the expected literal and actions' head matcher.
  /// This is pre-calculated to optimize the runtime performance.
  kind_literal_map: HashMap<&'static TokenKindId<Kind>, LiteralMap<Kind, ActionState, ErrorType>>,
}

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  /// Create a new [`StatelessLexer`] from a list of actions.
  /// This function will pre-calculate some collections to optimize the runtime performance.
  pub fn new(actions: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    let actions = actions.into_iter().map(|a| a.into_general()).collect();

    // known kinds => actions
    let kinds_action_map = Self::init_kind_map(&actions);

    // collect known chars/literals using all actions so we can re-use these map for all head/literal maps
    let known_head_chars = HeadMap::collect_all_known(&actions);
    let known_literals = LiteralMap::collect_all_known(&actions);

    Self {
      kind_head_map: kinds_action_map
        .iter()
        .map(|(k, v)| (*k, HeadMap::new(v, known_head_chars.clone())))
        .collect(),
      kind_literal_map: kinds_action_map
        .iter()
        .map(|(k, v)| {
          (
            *k,
            LiteralMap::new(v, known_literals.clone(), &known_head_chars),
          )
        })
        .collect(),
      literal_map: LiteralMap::new(&actions, known_literals, &known_head_chars),
      head_map: HeadMap::new(&actions, known_head_chars),
    }
  }

  #[inline] // there is only one call site, so mark this as inline
  fn init_kind_map(
    actions: &Vec<GeneralAction<Kind, ActionState, ErrorType>>,
  ) -> HashMap<&'static TokenKindId<Kind>, Vec<GeneralAction<Kind, ActionState, ErrorType>>> {
    let mut res = HashMap::new();
    // prepare kind map, add value for all known possible kinds
    // this has to be done before filling the map
    // because we need to iter over all possible kinds when filling the map
    for a in actions {
      res.entry(a.kind()).or_insert(Vec::new());
    }
    // fill it
    for a in actions {
      // TODO: why the following line is not covered in the coverage report?
      if a.muted() {
        // muted, add to all kinds
        for (_, vec) in res.iter_mut() {
          vec.push(a.clone());
        }
      } else {
        // non-muted, only add to possible kinds
        // SAFETY: the entry is guaranteed to exist since we've collected all possible kinds
        unsafe { res.get_mut(a.kind()).unwrap_unchecked() }.push(a.clone());
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    res
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
    let literal_a_head_map = &lexer.literal_map.known_map().get("a").unwrap();
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
    let literal_a_muted_head_map = &lexer.literal_map.muted_map();
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
      &kind_a_literal_map.known_map()["a"].known_map()[&'a'],
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
      &kind_a_literal_map.known_map()["a"].known_map()[&'b'],
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
      &kind_a_literal_map.known_map()["a"].known_map()[&'c'],
      vec![
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_literal_map.known_map()["a"].unknown_fallback(),
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
      &kind_a_literal_map.muted_map().known_map()[&'a'],
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
      &kind_a_literal_map.muted_map().known_map()[&'b'],
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
      &kind_a_literal_map.muted_map().known_map()[&'c'],
      vec![
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_actions_eq(
      &kind_a_literal_map.muted_map().unknown_fallback(),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").unchecked_head_unknown().mute().bind(A),  // A, Unknown, muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").unchecked_head_unknown().mute().bind(B),  // B, Unknown, muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
  }
}
