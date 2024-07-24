use super::head_map::HeadMap;
use crate::lexer::action::Action;
use std::{collections::HashMap, rc::Rc};

pub(super) struct LiteralMapItem<Kind: 'static, ActionState, ErrorType> {
  pub head_map: HeadMap<Kind, ActionState, ErrorType>,
  pub muted_head_map: HeadMap<Kind, ActionState, ErrorType>,
}

pub(super) struct LiteralMap<Kind: 'static, ActionState, ErrorType> {
  /// The key of the map is the literal.
  known_map: HashMap<String, LiteralMapItem<Kind, ActionState, ErrorType>>,
  // for literal map there is no unknown_fallback because we don't check
  // actions with mismatched/unknown literals (should panic)
}

impl<Kind, ActionState, ErrorType> LiteralMap<Kind, ActionState, ErrorType> {
  /// Collect all known literals from all actions instead of a subset of actions to make sure
  /// 'known' as a consistent meaning across all literal maps in a stateless lexer
  /// (otherwise maybe only a subset of literals are known for a subset of actions,
  /// in this case the 'known' has an inconsistent meaning).
  /// This must be done before creating a literal map because we need to iter over all known literals
  /// when filling the literal map with no-literal actions.
  pub fn collect_all_known(
    actions: &Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  ) -> HashMap<String, Vec<Rc<Action<Kind, ActionState, ErrorType>>>> {
    let mut res = HashMap::new();

    for a in actions {
      if let Some(literal) = a.literal() {
        res.entry(literal.clone()).or_insert(Vec::new());
      }
    }

    res
  }

  /// Create a self with a subset of actions, a known literal map created by [`Self::collect_all_known`]
  /// and a known head map created by [`HeadMap::collect_all_known`].
  pub fn new(
    actions: &Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
    mut known_map: HashMap<String, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
    known_head_map: &HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
  ) -> Self {
    // fill the action map
    for a in actions {
      // [[check if the action is muted or not in literal map]]
      if a.muted() {
        // muted, expectation.literal will be ignored, add to all known literals
        for vec in known_map.values_mut() {
          vec.push(a.clone());
        }
        // ignore self.literal, just continue
        continue;
      }

      if let Some(literal) = a.literal() {
        known_map.get_mut(literal).unwrap().push(a.clone());
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    Self {
      known_map: known_map
        .into_iter()
        .map(|(literal, vec)| {
          (
            literal,
            LiteralMapItem {
              head_map: HeadMap::new(&vec, known_head_map.clone()),
              muted_head_map: HeadMap::new(
                &vec.into_iter().filter(|a| a.muted()).collect(),
                known_head_map.clone(),
              ),
            },
          )
        })
        .collect(),
    }
  }

  pub fn known_map(&self) -> &HashMap<String, LiteralMapItem<Kind, ActionState, ErrorType>> {
    &self.known_map
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::{exact, regex},
    token::MockTokenKind,
  };

  fn r<S: 'static, E>(s: &str) -> Action<MockTokenKind<()>, S, E> {
    regex(s)
  }

  fn assert_actions_eq(
    actions: &Vec<Rc<Action<MockTokenKind<()>>>>,
    expected: Vec<Action<MockTokenKind<()>>>,
  ) {
    assert_eq!(actions.len(), expected.len());
    for i in 0..actions.len() {
      assert_eq!(actions[i].kind(), expected[i].kind());
      assert_eq!(actions[i].head_matcher(), expected[i].head_matcher());
      assert_eq!(actions[i].literal(), expected[i].literal());
      assert_eq!(actions[i].muted(), expected[i].muted());
    }
  }

  #[test]
  fn test_literal_map() {
    let actions: Vec<Rc<Action<MockTokenKind<()>>>> = vec![
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
    .map(Rc::new)
    .collect();

    let lm = LiteralMap::new(
      &actions,
      LiteralMap::collect_all_known(&actions),
      &HeadMap::collect_all_known(&actions),
    );

    // collect all literals
    assert_eq!(lm.known_map().len(), ["a", "b"].len());

    let literal_a_head_map = &lm.known_map().get("a").unwrap().head_map;
    assert_eq!(literal_a_head_map.known_map().len(), ['a', 'b', 'c'].len());
    assert_actions_eq(
      literal_a_head_map.known_map().get(&'a').unwrap(),
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
    assert_actions_eq(
      literal_a_head_map.known_map().get(&'b').unwrap(),
      vec![
        r("a").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("a").mute(),                           // no head, muted
        exact("b").mute(),                       // "b", muted
        r("b").unchecked_head_in(['b']).mute(),  // OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("b").mute(),                           // no head, muted
      ],
    );
    assert_actions_eq(
      literal_a_head_map.known_map().get(&'c').unwrap(),
      vec![
        r("a").mute(), // no head, muted
        r("b").mute(), // no head, muted
      ],
    );
    assert_actions_eq(
      literal_a_head_map.unknown_fallback(),
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

    let literal_a_muted_head_map = &lm.known_map().get("a").unwrap().muted_head_map;
    assert_eq!(
      literal_a_muted_head_map.known_map().len(),
      ['a', 'b', 'c'].len()
    );
    assert_actions_eq(
      literal_a_muted_head_map.known_map().get(&'a').unwrap(),
      vec![
        exact("a").mute(),                       // "a", muted
        r("a").unchecked_head_in(['a']).mute(),  // OneOf('a'), muted
        r("a").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("a").mute(),                           // no head, muted
        r("b").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("b").mute(),                           // no head, muted
      ],
    );
    assert_actions_eq(
      literal_a_muted_head_map.known_map().get(&'b').unwrap(),
      vec![
        r("a").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("a").mute(),                           // no head, muted
        exact("b").mute(),                       // "b", muted
        r("b").unchecked_head_in(['b']).mute(),  // OneOf('b'), muted
        r("b").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("b").mute(),                           // no head, muted
      ],
    );
    assert_actions_eq(
      literal_a_muted_head_map.known_map().get(&'c').unwrap(),
      vec![
        r("a").mute(), // no head, muted
        r("b").mute(), // no head, muted
      ],
    );
    assert_actions_eq(
      literal_a_muted_head_map.unknown_fallback(),
      vec![
        r("a").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("a").unchecked_head_unknown().mute(),  // Unknown, muted
        r("a").mute(),                           // no head, muted
        r("b").unchecked_head_not(['c']).mute(), // Not('c'), muted
        r("b").unchecked_head_unknown().mute(),  // Unknown, muted
        r("b").mute(),                           // no head, muted
      ],
    );
    // literal_b_muted_head_map should be similar to literal_a_muted_head_map, skip
  }
}
