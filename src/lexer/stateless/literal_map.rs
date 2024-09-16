use super::head_map::{HeadMap, KnownHeadChars};
use crate::lexer::action::RcAction;
use std::collections::HashMap;

#[derive(Debug)]
pub(super) struct LiteralMap<'a, Kind, State, Heap> {
  /// The key of the map is the literal.
  /// Actions in the value should be either muted or have a matched
  /// [`Action::literal`](crate::lexer::action::Action::literal)
  known_map: HashMap<String, HeadMap<'a, Kind, State, Heap>>, // TODO: optimize using lookup table if the literal's first char is unique
  /// When the rest of the input text doesn't starts with the expected literal,
  /// only muted actions will be checked.
  muted_map: HeadMap<'a, Kind, State, Heap>,
  // for literal map there is no unknown_fallback because we don't check
  // actions with mismatched/unknown literals (should panic)
}

/// A new-type to represent the return type of [`LiteralMap::collect_all_known`].
/// This is to prevent other modules from modifying the known map by mistake
/// before calling [`LiteralMap::new`].
pub(super) struct KnownLiterals<'a, Kind, State, Heap>(
  HashMap<String, Vec<RcAction<'a, Kind, State, Heap>>>,
);

impl<'a, Kind, State, Heap> Clone for KnownLiterals<'a, Kind, State, Heap> {
  #[inline]
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<'a, Kind, State, Heap> LiteralMap<'a, Kind, State, Heap> {
  /// Collect all known literals from all actions instead of a subset of actions to make sure
  /// 'known' as a consistent meaning across all literal maps in a stateless lexer
  /// (otherwise maybe only a subset of literals are known for a subset of actions,
  /// in which case the 'known' has an inconsistent meaning).
  /// This must be done before creating a literal map because we need to iter over all known literals
  /// when filling the literal map with no-literal actions.
  #[inline] // there is only one call site, so mark this as inline
  pub fn collect_all_known(
    actions: &[RcAction<'a, Kind, State, Heap>],
  ) -> KnownLiterals<'a, Kind, State, Heap> {
    let mut res = HashMap::new();

    for (_, p) in actions {
      if let Some(literal) = p.literal() {
        res.entry(literal.clone()).or_insert(Vec::new());
      }
    }

    KnownLiterals(res)
  }

  /// Create a self with a subset of actions, a known literal map created by [`Self::collect_all_known`]
  /// and a known head map created by [`HeadMap::collect_all_known`].
  pub fn new(
    // TODO: accept iter instead of slice to prevent unnecessary allocation
    actions: &[RcAction<'a, Kind, State, Heap>],
    known_map: KnownLiterals<'a, Kind, State, Heap>,
    known_head_map: &KnownHeadChars<'a, Kind, State, Heap>,
  ) -> Self {
    let mut known_map = known_map.0;
    // fill the action map
    for (e, p) in actions {
      if p.muted() {
        // muted, expectation.literal will be ignored, add to all known literals
        for a in known_map.values_mut() {
          a.push((e.clone(), p.clone()));
        }
        // ignore self.literal, just continue
        continue;
      }

      // else, not muted, check literal
      if let Some(literal) = p.literal() {
        // SAFETY: the key must exist because we have collected all known chars in `collect_all_known`
        // and `KnownLiterals` ensures the known map is not modified before creating the literal map
        let a = unsafe { known_map.get_mut(literal).unwrap_unchecked() };
        a.push((e.clone(), p.clone()));
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    let muted_actions: Vec<_> = actions
      .iter()
      .filter(|(_, p)| p.muted())
      .map(|(e, p)| (e.clone(), p.clone()))
      .collect();
    Self {
      known_map: known_map
        .into_iter()
        .map(|(literal, actions)| (literal, HeadMap::new(&actions, known_head_map.clone())))
        .collect(),
      muted_map: HeadMap::new(&muted_actions, known_head_map.clone()),
    }
  }

  #[inline]
  pub const fn known_map(&self) -> &HashMap<String, HeadMap<Kind, State, Heap>> {
    &self.known_map
  }

  #[inline]
  pub const fn muted_map(&self) -> &HeadMap<Kind, State, Heap> {
    &self.muted_map
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    kind::MockKind,
    lexer::{
      action::{exact, regex, Action},
      stateless::head_map::RuntimeActions,
    },
  };

  fn r<S>(s: &str) -> Action<MockKind<()>, S> {
    regex(s)
  }

  fn assert_immutable_actions_eq(
    actions: &RuntimeActions<MockKind<()>, (), ()>,
    expected: Vec<Action<MockKind<()>, ()>>,
  ) {
    assert_eq!(actions.len(), expected.len());
    for (m, e) in actions.muted().iter().zip(expected.iter()) {
      assert_eq!(*m, e.muted());
    }
  }

  #[test]
  fn test_literal_map() {
    let actions: Vec<_> = vec![
      exact("a"),                              // "a", not muted
      exact("a").mute(),                       // "a", muted
      r("a").unchecked_head_in(['a']),         // OneOf('a'), not muted
      r("a").unchecked_head_in(['a']).mute(),  // OneOf('a'), muted
      r("a").unchecked_head_not(['c']),        // Not('c'), not muted
      r("a").unchecked_head_not(['c']).mute(), // Not('c'), muted
      r("a").unchecked_head_unknown(),         // Unknown, not muted
      r("a").unchecked_head_unknown().mute(),  // Unknown, muted
      r("a"),                                  // no head, not muted
      r("a").mute(),                           // no head, muted
      exact("b"),                              // "b", not muted
      exact("b").mute(),                       // "b", muted
      r("b").unchecked_head_in(['b']),         // OneOf('b'), not muted
      r("b").unchecked_head_in(['b']).mute(),  // OneOf('b'), muted
      r("b").unchecked_head_not(['c']),        // Not('c'), not muted
      r("b").unchecked_head_not(['c']).mute(), // Not('c'), muted
      r("b").unchecked_head_unknown(),         // Unknown, not muted
      r("b").unchecked_head_unknown().mute(),  // Unknown, muted
      r("b"),                                  // no head, not muted
      r("b").mute(),                           // no head, muted
    ]
    .into_iter()
    .map(|a| a.into_rc())
    .collect();

    let lm = LiteralMap::new(
      &actions,
      LiteralMap::collect_all_known(&actions),
      &HeadMap::collect_all_known(&actions),
    );

    // collect all literals
    assert_eq!(lm.known_map().len(), ["a", "b"].len());

    let literal_a_head_map = &lm.known_map().get("a").unwrap();
    assert_immutable_actions_eq(
      literal_a_head_map.get('a'),
      vec![
        exact("a"),                              // "a", not muted
        exact("a").mute(),                       // "a", muted
        r("a").unchecked_head_in(['a']).mute(),  // OneOf('a'), muted
        r("a").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("a").mute(),                           // no head, muted
        r("b").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("b").mute(),                           // no head, muted
      ],
    );
    assert_immutable_actions_eq(
      literal_a_head_map.get('b'),
      vec![
        r("a").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("a").mute(),                           // no head, muted
        exact("b").mute(),                       // "b", muted
        r("b").unchecked_head_in(['b']).mute(),  // OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("b").mute(),                           // no head, muted
      ],
    );
    assert_immutable_actions_eq(
      literal_a_head_map.get('c'),
      vec![
        r("a").mute(), // no head, muted
        r("b").mute(), // no head, muted
      ],
    );
    assert_immutable_actions_eq(
      literal_a_head_map.get('z'),
      vec![
        r("a").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("a").unchecked_head_unknown().mute(),  // Unknown, muted
        r("a").mute(),                           // no head, muted
        r("b").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("b").unchecked_head_unknown().mute(),  // Unknown, muted
        r("b").mute(),                           // no head, muted
      ],
    );
    // literal_b_head_map should be similar to literal_a_head_map, skip

    let muted_map = &lm.muted_map();
    assert_immutable_actions_eq(
      muted_map.get('a'),
      vec![
        exact("a").mute(),                       // "a", muted
        r("a").unchecked_head_in(['a']).mute(),  // OneOf('a'), muted
        r("a").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("a").mute(),                           // no head, muted
        r("b").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("b").mute(),                           // no head, muted
      ],
    );
    assert_immutable_actions_eq(
      muted_map.get('b'),
      vec![
        r("a").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("a").mute(),                           // no head, muted
        exact("b").mute(),                       // "b", muted
        r("b").unchecked_head_in(['b']).mute(),  // OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("b").mute(),                           // no head, muted
      ],
    );
    assert_immutable_actions_eq(
      muted_map.get('c'),
      vec![
        r("a").mute(), // no head, muted
        r("b").mute(), // no head, muted
      ],
    );
    assert_immutable_actions_eq(
      muted_map.get('z'),
      vec![
        r("a").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("a").unchecked_head_unknown().mute(),  // Unknown, muted
        r("a").mute(),                           // no head, muted
        r("b").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("b").unchecked_head_unknown().mute(),  // Unknown, muted
        r("b").mute(),                           // no head, muted
      ],
    );
  }
}
