use crate::lexer::action::{GeneralAction, HeadMatcher, ImmutableAction};
use std::{collections::HashMap, rc::Rc};

/// [`HeadMapActions`] consists of 2 parts:
/// - [`Self::immutables`]: immutable actions, this should always be checked first.
/// - [`Self::rest`]: immutable or mutable actions, this should be checked after [`Self::immutables`].
/// If this is not empty, this must starts with a mutable action.
pub(super) struct HeadMapActions<Kind: 'static, State, ErrorType> {
  immutables: Vec<Rc<ImmutableAction<Kind, State, ErrorType>>>,
  rest: Vec<GeneralAction<Kind, State, ErrorType>>,
}

impl<Kind: 'static, State, ErrorType> Clone for HeadMapActions<Kind, State, ErrorType> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      immutables: self.immutables.clone(),
      rest: self.rest.clone(),
    }
  }
}

impl<Kind, State, ErrorType> HeadMapActions<Kind, State, ErrorType> {
  #[inline]
  pub const fn new() -> Self {
    Self {
      immutables: Vec::new(),
      rest: Vec::new(),
    }
  }

  #[inline]
  pub fn push(&mut self, action: GeneralAction<Kind, State, ErrorType>) {
    if self.rest.len() == 0 {
      // no mutable actions yet, check if the action is immutable
      match action {
        GeneralAction::Immutable(immutable) => self.immutables.push(immutable),
        GeneralAction::Mutable(_) => self.rest.push(action),
      }
    } else {
      // mutable actions are already added, add the action to the rest
      self.rest.push(action);
    }
  }

  // getters
  #[inline]
  pub const fn immutables(&self) -> &Vec<Rc<ImmutableAction<Kind, State, ErrorType>>> {
    &self.immutables
  }
  #[inline]
  pub const fn rest(&self) -> &Vec<GeneralAction<Kind, State, ErrorType>> {
    &self.rest
  }

  // TODO: remove this function?
  #[inline]
  pub fn len(&self) -> usize {
    self.immutables.len() + self.rest.len()
  }
}

pub(super) struct HeadMap<Kind: 'static, State, ErrorType> {
  /// Store actions for known chars.
  known_map: HashMap<char, HeadMapActions<Kind, State, ErrorType>>,
  /// Store actions for unknown chars.
  unknown_fallback: HeadMapActions<Kind, State, ErrorType>,
}

/// A new-type to represent the return type of [`HeadMap::collect_all_known`].
/// This is to prevent other modules from modifying the known map by mistake
/// before calling [`HeadMap::new`].
pub(super) struct KnownHeadChars<Kind: 'static, State, ErrorType>(
  HashMap<char, HeadMapActions<Kind, State, ErrorType>>,
);

impl<Kind: 'static, State, ErrorType> Clone for KnownHeadChars<Kind, State, ErrorType> {
  #[inline]
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<Kind, State, ErrorType> HeadMap<Kind, State, ErrorType> {
  /// Collect all known head chars from all actions instead of a subset of actions to make sure
  /// 'known' has a consistent meaning across all head maps in a stateless lexer
  /// (otherwise maybe only a subset of chars are known for a subset of actions,
  /// in this case the 'known' has an inconsistent meaning).
  /// This must be done before creating a head map because we need to iter over all known chars when filling the head map
  /// with [`HeadMatcher::Not`] and [`HeadMatcher::Unknown`].
  #[inline] // there is only one call site, so mark this as inline
  pub fn collect_all_known(
    actions: &Vec<GeneralAction<Kind, State, ErrorType>>,
  ) -> KnownHeadChars<Kind, State, ErrorType> {
    let mut res = HashMap::new();

    for a in actions {
      if let Some(head) = a.head() {
        for c in match head {
          HeadMatcher::OneOf(set) | HeadMatcher::Not(set) => set,
          HeadMatcher::Unknown => continue,
        } {
          res.entry(*c).or_insert(HeadMapActions::new());
        }
      }
    }

    KnownHeadChars(res)
  }

  /// Create a new instance with a subset of actions and a known char map created by [`Self::collect_all_known`].
  pub fn new(
    actions: &Vec<GeneralAction<Kind, State, ErrorType>>,
    known_map: KnownHeadChars<Kind, State, ErrorType>,
  ) -> Self {
    let mut res = Self {
      known_map: known_map.0,
      unknown_fallback: HeadMapActions::new(),
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

  /// Get actions by the next char.
  #[inline]
  pub fn get(&self, next: char) -> &HeadMapActions<Kind, State, ErrorType> {
    self.known_map.get(&next).unwrap_or(&self.unknown_fallback)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::{exact, regex, Action},
    token::MockTokenKind,
  };

  #[test]
  fn test_head_map_actions() {
    let mut actions: HeadMapActions<MockTokenKind<()>, i32, ()> = HeadMapActions::new();
    assert_eq!(actions.len(), 0);

    actions.push(exact("a").into_general());
    assert_eq!(actions.len(), 1);
    assert_eq!(actions.immutables.len(), 1);

    actions.push(exact("b").into_general());
    assert_eq!(actions.len(), 2);
    assert_eq!(actions.immutables.len(), 2);

    actions.push(exact("c").prepare(|input| *input.state += 1).into_general());
    assert_eq!(actions.len(), 3);
    assert_eq!(actions.immutables.len(), 2);
    assert_eq!(actions.rest.len(), 1);

    actions.push(exact("d").into_general());
    assert_eq!(actions.len(), 4);
    assert_eq!(actions.immutables.len(), 2);
    assert_eq!(actions.rest.len(), 2);

    actions.push(exact("e").prepare(|input| *input.state += 1).into_general());
    assert_eq!(actions.len(), 5);
    assert_eq!(actions.immutables.len(), 2);
    assert_eq!(actions.rest.len(), 3);
  }

  fn assert_immutable_actions_eq(
    actions: &HeadMapActions<MockTokenKind<()>, (), ()>,
    expected: Vec<Action<MockTokenKind<()>, (), ()>>,
  ) {
    assert_eq!(actions.len(), expected.len());
    for i in 0..actions.immutables.len() {
      assert_eq!(actions.immutables[i].kind(), expected[i].kind());
      assert_eq!(actions.immutables[i].head(), expected[i].head());
      assert_eq!(actions.immutables[i].literal(), expected[i].literal());
      assert_eq!(actions.immutables[i].muted(), expected[i].muted());
    }
  }

  #[test]
  fn test_head_map() {
    let actions: Vec<GeneralAction<MockTokenKind<()>, (), ()>> = vec![
      exact("a"),
      exact("aa"),
      exact("b"),
      regex("[^c]").unchecked_head_not(['c']),
      regex(".").unchecked_head_unknown(),
      regex("a_muted").unchecked_head_in(['a']).mute(),
      regex("no_head"),
    ]
    .into_iter()
    .map(|a| a.into_general())
    .collect();

    let hm = HeadMap::new(&actions, HeadMap::collect_all_known(&actions));

    // collect all known heads
    assert!(hm.known_map.contains_key(&'a'));
    assert!(hm.known_map.contains_key(&'b'));
    assert!(hm.known_map.contains_key(&'c'));
    assert_eq!(hm.known_map.len(), 3);

    // check actions
    assert_immutable_actions_eq(
      &hm.get('a'),
      vec![
        exact("a"),
        exact("aa"),
        regex("[^c]").unchecked_head_not(['c']),
        regex("a_muted").unchecked_head_in(['a']).mute(),
        regex("no_head"),
      ],
    );
    assert_immutable_actions_eq(
      &hm.get('b'),
      vec![
        exact("b"),
        regex("[^c]").unchecked_head_not(['c']),
        regex("no_head"),
      ],
    );
    assert_immutable_actions_eq(&hm.get('c'), vec![regex("no_head")]);
    assert_immutable_actions_eq(
      &hm.get('z'),
      vec![
        regex("[^c]").unchecked_head_not(['c']),
        regex(".").unchecked_head_unknown(),
        regex("no_head"),
      ],
    );
  }
}
