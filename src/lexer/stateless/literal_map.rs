use super::head_map::{HeadMap, KnownHeadChars};
use crate::lexer::action::{RcActionExec, RcActionProps};
use std::collections::HashMap;

pub(super) struct LiteralMap<Kind, State, ErrorType> {
  /// The key of the map is the literal.
  /// Actions in the value should be either muted or have a matched
  /// [`Action::literal`](crate::lexer::action::Action::literal)
  known_map: HashMap<String, HeadMap<Kind, State, ErrorType>>, // TODO: optimize using lookup table
  /// When the rest of the input text doesn't starts with the expected literal,
  /// only muted actions will be checked.
  muted_map: HeadMap<Kind, State, ErrorType>,
  // for literal map there is no unknown_fallback because we don't check
  // actions with mismatched/unknown literals (should panic)
}

/// A new-type to represent the return type of [`LiteralMap::collect_all_known`].
/// This is to prevent other modules from modifying the known map by mistake
/// before calling [`LiteralMap::new`].
pub(super) struct KnownLiterals<Kind, State, ErrorType>(
  HashMap<
    String,
    (
      Vec<RcActionExec<Kind, State, ErrorType>>,
      Vec<RcActionProps<Kind>>,
    ),
  >,
);

impl<Kind, State, ErrorType> Clone for KnownLiterals<Kind, State, ErrorType> {
  #[inline]
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<Kind, State, ErrorType> LiteralMap<Kind, State, ErrorType> {
  /// Collect all known literals from all actions instead of a subset of actions to make sure
  /// 'known' as a consistent meaning across all literal maps in a stateless lexer
  /// (otherwise maybe only a subset of literals are known for a subset of actions,
  /// in this case the 'known' has an inconsistent meaning).
  /// This must be done before creating a literal map because we need to iter over all known literals
  /// when filling the literal map with no-literal actions.
  #[inline] // there is only one call site, so mark this as inline
  pub fn collect_all_known(
    props: &Vec<RcActionProps<Kind>>,
  ) -> KnownLiterals<Kind, State, ErrorType> {
    let mut res = HashMap::new();

    for p in props {
      if let Some(literal) = p.literal() {
        res
          .entry(literal.clone())
          .or_insert((Vec::new(), Vec::new()));
      }
    }

    KnownLiterals(res)
  }

  /// Create a self with a subset of actions, a known literal map created by [`Self::collect_all_known`]
  /// and a known head map created by [`HeadMap::collect_all_known`].
  pub fn new(
    execs: &Vec<RcActionExec<Kind, State, ErrorType>>,
    props: &Vec<RcActionProps<Kind>>,
    known_map: KnownLiterals<Kind, State, ErrorType>,
    known_head_map: &KnownHeadChars<Kind, State, ErrorType>,
  ) -> Self {
    let mut known_map = known_map.0;
    // fill the action map
    for (e, p) in execs.iter().zip(props.iter()) {
      if p.muted() {
        // muted, expectation.literal will be ignored, add to all known literals
        for (execs, props) in known_map.values_mut() {
          execs.push(e.clone());
          props.push(p.clone());
        }
        // ignore self.literal, just continue
        continue;
      }

      // else, not muted, check literal
      if let Some(literal) = p.literal() {
        // SAFETY: the key must exist because we have collected all known chars in `collect_all_known`
        // and `KnownLiterals` ensures the known map is not modified before creating the literal map
        let (execs, props) = unsafe { known_map.get_mut(literal).unwrap_unchecked() };
        execs.push(e.clone());
        props.push(p.clone());
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    let (muted_execs, muted_props) = execs
      .iter()
      .zip(props.iter())
      .filter(|(_, p)| p.muted())
      .map(|(e, p)| (e.clone(), p.clone()))
      .unzip();
    Self {
      known_map: known_map
        .into_iter()
        .map(|(literal, (execs, props))| {
          (
            literal,
            HeadMap::new(&execs, &props, known_head_map.clone()),
          )
        })
        .collect(),
      muted_map: HeadMap::new(&muted_execs, &muted_props, known_head_map.clone()),
    }
  }

  #[inline]
  pub const fn known_map(&self) -> &HashMap<String, HeadMap<Kind, State, ErrorType>> {
    &self.known_map
  }

  #[inline]
  pub const fn muted_map(&self) -> &HeadMap<Kind, State, ErrorType> {
    &self.muted_map
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::{exact, regex, Action},
    stateless::head_map::HeadMapActions,
    token::MockTokenKind,
  };

  fn r<S, E>(s: &str) -> Action<MockTokenKind<()>, S, E> {
    regex(s)
  }

  fn assert_immutable_actions_eq(
    actions: &HeadMapActions<MockTokenKind<()>, (), ()>,
    expected: Vec<Action<MockTokenKind<()>, (), ()>>,
  ) {
    assert_eq!(actions.len(), expected.len());
    for i in 0..actions.immutables().len() {
      assert_eq!(actions.immutables().muted()[i], expected[i].muted());
    }
  }

  #[test]
  fn test_literal_map() {
    let (execs, props) = vec![
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
    .unzip();

    let lm = LiteralMap::new(
      &execs,
      &props,
      LiteralMap::collect_all_known(&props),
      &HeadMap::collect_all_known(&props),
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
