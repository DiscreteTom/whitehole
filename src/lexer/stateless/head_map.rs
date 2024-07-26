use crate::lexer::action::{Action, HeadMatcher};
use std::{collections::HashMap, rc::Rc};

pub(super) struct HeadMap<Kind: 'static, ActionState, ErrorType> {
  /// Store actions for known chars.
  known_map: HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
  /// Store actions for unknown chars.
  unknown_fallback: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
}

/// A new-type to represent the return type of [`HeadMap::collect_all_known`].
/// This is to prevent other modules from modifying the known map by mistake
/// before calling [`HeadMap::new`].
pub(super) struct KnownHeadChars<Kind: 'static, ActionState, ErrorType>(
  HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
);

impl<Kind: 'static, ActionState, ErrorType> Clone for KnownHeadChars<Kind, ActionState, ErrorType> {
  #[inline]
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<Kind, ActionState, ErrorType> HeadMap<Kind, ActionState, ErrorType> {
  /// Collect all known head chars from all actions instead of a subset of actions to make sure
  /// 'known' has a consistent meaning across all head maps in a stateless lexer
  /// (otherwise maybe only a subset of chars are known for a subset of actions,
  /// in this case the 'known' has an inconsistent meaning).
  /// This must be done before creating a head map because we need to iter over all known chars when filling the head map
  /// with [`HeadMatcher::Not`] and [`HeadMatcher::Unknown`].
  #[inline] // there is only one call site, so mark this as inline
  pub fn collect_all_known(
    actions: &Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  ) -> KnownHeadChars<Kind, ActionState, ErrorType> {
    let mut res = HashMap::new();

    for a in actions {
      if let Some(head) = a.head() {
        for c in match head {
          HeadMatcher::OneOf(set) | HeadMatcher::Not(set) => set,
          HeadMatcher::Unknown => continue,
        } {
          res.entry(*c).or_insert(Vec::new());
        }
      }
    }

    KnownHeadChars(res)
  }

  /// Create a new instance with a subset of actions and a known char map created by [`Self::collect_all_known`].
  pub fn new(
    actions: &Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
    known_map: KnownHeadChars<Kind, ActionState, ErrorType>,
  ) -> Self {
    let mut res = Self {
      known_map: known_map.0,
      unknown_fallback: Vec::new(),
    };

    // fill the head map
    for a in actions {
      // when lexing the lexer needs to check the head matcher no matter the action is muted or not
      // so we won't check if the action is muted here
      if let Some(head) = a.head() {
        // TODO: why the following line is not covered in the coverage report?
        match head {
          HeadMatcher::OneOf(set) => {
            for c in set {
              // SAFETY: the key must exist because we have collected all known chars in `collect_all_known`
              // and `KnownHeadChars` ensures the known map is not modified before creating the head map
              unsafe { res.known_map.get_mut(c).unwrap_unchecked() }.push(a.clone());
            }
          }
          HeadMatcher::Not(set) => {
            // e.g. the head matcher is `Not(['a', 'b'])`, the `set` is `['a', 'b']`
            for (c, vec) in res.known_map.iter_mut() {
              // e.g. if the head char is `'c'` which is not in `set`, add the action to the vec
              if !set.contains(c) {
                vec.push(a.clone());
              }
            }
            res.unknown_fallback.push(a.clone());
          }
          HeadMatcher::Unknown => {
            res.unknown_fallback.push(a.clone());
          }
        }
      } else {
        // no head matcher, add the action to all known chars
        for vec in res.known_map.values_mut() {
          vec.push(a.clone());
        }
        // and unknown fallback
        res.unknown_fallback.push(a.clone());
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    res
  }

  #[inline]
  pub const fn known_map(&self) -> &HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>> {
    &self.known_map
  }
  #[inline]
  pub const fn unknown_fallback(&self) -> &Vec<Rc<Action<Kind, ActionState, ErrorType>>> {
    &self.unknown_fallback
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::{exact, regex},
    token::MockTokenKind,
  };

  fn assert_actions_eq(
    actions: &Vec<Rc<Action<MockTokenKind<()>>>>,
    expected: Vec<Action<MockTokenKind<()>>>,
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
  fn test_head_map() {
    let actions: Vec<Rc<Action<MockTokenKind<()>>>> = vec![
      exact("a"),
      exact("aa"),
      exact("b"),
      regex("[^c]").unchecked_head_not(['c']),
      regex(".").unchecked_head_unknown(),
      regex("a_muted").unchecked_head_in(['a']).mute(),
      regex("no_head"),
    ]
    .into_iter()
    .map(Rc::new)
    .collect();

    let hm = HeadMap::new(&actions, HeadMap::collect_all_known(&actions));

    // collect all known heads
    assert!(hm.known_map().contains_key(&'a'));
    assert!(hm.known_map().contains_key(&'b'));
    assert!(hm.known_map().contains_key(&'c'));
    assert_eq!(hm.known_map().len(), 3);

    // check actions
    assert_actions_eq(
      &hm.known_map()[&'a'],
      vec![
        exact("a"),
        exact("aa"),
        regex("[^c]").unchecked_head_not(['c']),
        regex("a_muted").unchecked_head_in(['a']).mute(),
        regex("no_head"),
      ],
    );
    assert_actions_eq(
      &hm.known_map()[&'b'],
      vec![
        exact("b"),
        regex("[^c]").unchecked_head_not(['c']),
        regex("no_head"),
      ],
    );
    assert_actions_eq(&hm.known_map()[&'c'], vec![regex("no_head")]);
    assert_actions_eq(
      hm.unknown_fallback(),
      vec![
        regex("[^c]").unchecked_head_not(['c']),
        regex(".").unchecked_head_unknown(),
        regex("no_head"),
      ],
    );
  }
}
