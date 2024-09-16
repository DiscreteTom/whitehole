//! Stateless, immutable lexer.
//!
//! ## Design
//!
//! [`StatelessLexer`] doesn't hold text, progress or states.
//! It is just a collection of immutable [`Action`]s, and it is immutable itself.
//! You can wrap it with [`Rc`](std::rc::Rc) to make it clone-able
//! and re-use it across multiple (stateful) lexers.
//!
//! The [`StatelessLexer`] implements all the core lexing features,
//! including peek, trim, expectation, fork, etc.
//! If you want a stateless experience, you can use the [`StatelessLexer`] directly,
//! but you may need to manage the text, progress and states manually.
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
//! then only actions accepting `'a'` as the first character will be executed.
//!
//! If an action has no head matcher, it will be executed no matter what the first character is.
//! So it is recommended to add a head matcher to all actions to make the lexer faster.
//!
//! (You can use [`LexerBuilder::ensure_head_matcher`](crate::lexer::builder::LexerBuilder::ensure_head_matcher)
//! to check if all actions have head matchers.)
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
//! When trimming, all non-muted actions are skipped, only muted actions will be executed.
//! Then the lexer will skip actions by the head matcher just like the case without expectation
//! during the lexing loop.
//!
//! ## Caveats
//!
//! Be careful if you lexer is stateful. Since in every lexing the evaluated actions
//! are different, you may need to manage the states carefully to avoid inconsistency.
//!
//! ## For developers
//!
//! Many of the modules have no exported items,
//! they are public just to make the following links work.
//! Read the source code if you are interested in how the stateless lexer works.
//! You don't need to read them if you just want to use the stateless lexer.
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

pub mod head_map;
pub mod lex;
pub mod literal_map;
pub mod options;
pub mod trim;
pub mod utils;

pub use options::*;

use super::action::{Action, RcActionExec, RcActionProps};
use crate::utils::lookup::{option::OptionLookupTable, Lookup};
use head_map::HeadMap;
use literal_map::LiteralMap;

/// Stateless, immutable lexer.
#[derive(Debug)]
pub struct StatelessLexer<'a, Kind, State = (), Heap = ()> {
  /// This is used to accelerate lexing by actions' head matcher when there is no expectation.
  /// This is pre-calculated to optimize the runtime performance.
  head_map: HeadMap<'a, Kind, State, Heap>,
  /// This is used to accelerate expected lexing by the expected kind and actions' head matcher.
  /// This is pre-calculated to optimize the runtime performance.
  kind_head_map: OptionLookupTable<HeadMap<'a, Kind, State, Heap>>,
  /// This is used to accelerate expected lexing by the expected literal and actions' head matcher.
  /// This is pre-calculated to optimize the runtime performance.
  literal_map: LiteralMap<'a, Kind, State, Heap>,
  /// This is used to accelerate expected lexing by the expected kind, the expected literal and actions' head matcher.
  /// This is pre-calculated to optimize the runtime performance.
  kind_literal_map: OptionLookupTable<LiteralMap<'a, Kind, State, Heap>>,
}

impl<'a, Kind, State, Heap> StatelessLexer<'a, Kind, State, Heap> {
  /// Create a new [`StatelessLexer`] from a list of actions.
  /// This function will pre-calculate some collections to optimize the runtime performance.
  pub fn new(actions: Vec<Action<'a, Kind, State, Heap>>) -> Self {
    // as per data oriented design, convert actions into 2 lists to optimize iteration efficiency (optimize CPU cache hit)
    let mut execs = Vec::with_capacity(actions.len());
    let mut props = Vec::with_capacity(actions.len());
    for a in actions {
      let (e, p) = a.into_rc();
      execs.push(e);
      props.push(p);
    }

    // known kinds => actions
    let kinds_action_map = Self::init_kind_map(&execs, &props);

    // collect known chars/literals using all actions so we can re-use these map for all head/literal maps
    let known_head_chars = HeadMap::collect_all_known(&props);
    let known_literals = LiteralMap::collect_all_known(&props);

    Self {
      kind_head_map: kinds_action_map
        .map_to_new(|(execs, props)| HeadMap::new(execs, props, known_head_chars.clone())),
      kind_literal_map: kinds_action_map.map_to_new(|(execs, props)| {
        LiteralMap::new(execs, props, known_literals.clone(), &known_head_chars)
      }),
      literal_map: LiteralMap::new(&execs, &props, known_literals, &known_head_chars),
      head_map: HeadMap::new(&execs, &props, known_head_chars),
    }
  }

  #[allow(clippy::type_complexity, reason = "this type only exists here once")]
  fn init_kind_map(
    execs: &[RcActionExec<'a, Kind, State, Heap>],
    props: &[RcActionProps<Kind>],
  ) -> OptionLookupTable<(
    Vec<RcActionExec<'a, Kind, State, Heap>>,
    Vec<RcActionProps<Kind>>,
  )> {
    let mut res = OptionLookupTable::with_keys_init(
      props.iter().map(|p| p.kind().value()),
      // in most cases there is only one action for each kind
      || (Vec::with_capacity(1), Vec::with_capacity(1)),
    );

    for (i, p) in props.iter().enumerate() {
      let e = unsafe { execs.get_unchecked(i) };
      if p.muted() {
        // muted, add to all kinds
        res.values_mut().for_each(|(execs, props)| {
          execs.push(e.clone());
          props.push(p.clone());
        });
      } else {
        // non-muted, only add to possible kinds
        // SAFETY: `p.kind().value()` is guaranteed to be in the range of `0..=max`
        let (execs, props) = unsafe { res.get_unchecked_mut(p.kind().value()) };
        execs.push(e.clone());
        props.push(p.clone());
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    res
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    kind::{MockKind, SubKind},
    lexer::action::{exact, regex},
    utils::lookup::Lookup,
  };
  use head_map::RuntimeActions;
  use whitehole_macros::_whitehole_kind;

  #[_whitehole_kind]
  #[derive(Debug, Clone)]
  pub enum MyKind {
    A,
    B,
  }

  fn r<S>(s: &str) -> Action<MockKind<()>, S> {
    regex(s)
  }

  fn assert_immutable_actions_eq(
    actions: &RuntimeActions<MyKind, (), ()>,
    expected: Vec<Action<MyKind, ()>>,
  ) {
    assert_eq!(actions.len(), expected.len());
    for (m, e) in actions.muted().iter().zip(expected.iter()) {
      assert_eq!(*m, e.muted());
    }
  }

  #[test]
  fn test_new_stateless_lexer() {
    let lexer: StatelessLexer<_> = StatelessLexer::new(vec![
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
    ]);

    // head_map
    assert_immutable_actions_eq(
      lexer.head_map.get('a'),
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
    // `lexer.head_map.get('b')` should be the similar to the above, skip.
    assert_immutable_actions_eq(
      lexer.head_map.get('c'),
      vec![
        r("a").bind(A),        // A, no head, not muted
        r("a").mute().bind(A), // A, no head, muted
        r("b").bind(B),        // B, no head, not muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      lexer.head_map.get('z'),
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
    let kind_a_head_map = lexer.kind_head_map.get(A::kind_id().value()).unwrap();
    assert_immutable_actions_eq(
      kind_a_head_map.get('a'),
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
    assert_immutable_actions_eq(
      kind_a_head_map.get('b'),
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
    assert_immutable_actions_eq(
      kind_a_head_map.get('c'),
      vec![
        r("a").bind(A),        // A, no head, not muted
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      kind_a_head_map.get('z'),
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
    let literal_a_head_map = &lexer.literal_map.known_map().get("a").unwrap();
    assert_immutable_actions_eq(
      literal_a_head_map.get('a'),
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
    assert_immutable_actions_eq(
      literal_a_head_map.get('b'),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        exact("b").mute().bind(B),                       // B, "b", muted
        r("b").unchecked_head_in(['b']).mute().bind(B),  // B, OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      literal_a_head_map.get('c'),
      vec![
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      literal_a_head_map.get('z'),
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
    assert_immutable_actions_eq(
      literal_a_muted_head_map.get('a'),
      vec![
        exact("a").mute().bind(A),                       // A, "a", muted
        r("a").unchecked_head_in(['a']).mute().bind(A),  // A, OneOf('a'), muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      literal_a_muted_head_map.get('b'),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        exact("b").mute().bind(B),                       // B, "b", muted
        r("b").unchecked_head_in(['b']).mute().bind(B),  // B, OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      literal_a_muted_head_map.get('c'),
      vec![
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      literal_a_muted_head_map.get('z'),
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
    let kind_a_literal_map = lexer.kind_literal_map.get(A::kind_id().value()).unwrap();
    assert_immutable_actions_eq(
      kind_a_literal_map.known_map()["a"].get('a'),
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
    assert_immutable_actions_eq(
      kind_a_literal_map.known_map()["a"].get('b'),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        exact("b").mute().bind(B),                       // B, "b", muted
        r("b").unchecked_head_in(['b']).mute().bind(B),  // B, OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      kind_a_literal_map.known_map()["a"].get('c'),
      vec![
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      kind_a_literal_map.known_map()["a"].get('z'),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").unchecked_head_unknown().mute().bind(A),  // A, Unknown, muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").unchecked_head_unknown().mute().bind(B),  // B, Unknown, muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      kind_a_literal_map.muted_map().get('a'),
      vec![
        exact("a").mute().bind(A),                       // A, "a", muted
        r("a").unchecked_head_in(['a']).mute().bind(A),  // A, OneOf('a'), muted
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      kind_a_literal_map.muted_map().get('b'),
      vec![
        r("a").unchecked_head_not(['c']).mute().bind(A), // A, Not('c'), muted
        r("a").mute().bind(A),                           // A, no head, muted
        exact("b").mute().bind(B),                       // B, "b", muted
        r("b").unchecked_head_in(['b']).mute().bind(B),  // B, OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute().bind(B), // B, Not('c'), muted
        r("b").mute().bind(B),                           // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      kind_a_literal_map.muted_map().get('c'),
      vec![
        r("a").mute().bind(A), // A, no head, muted
        r("b").mute().bind(B), // B, no head, muted
      ],
    );
    assert_immutable_actions_eq(
      kind_a_literal_map.muted_map().get('z'),
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
