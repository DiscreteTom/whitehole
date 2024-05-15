use crate::lexer::action::{Action, HeadMatcher};
use std::{collections::HashMap, rc::Rc};

pub(super) struct HeadMap<Kind: 'static, ActionState, ErrorType> {
  /// Store actions for known chars.
  known_map: HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
  /// Store actions for unknown chars.
  unknown_fallback: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
}

impl<Kind, ActionState, ErrorType> HeadMap<Kind, ActionState, ErrorType> {
  /// Collect all known head chars from all actions instead of a subset of actions to make sure
  /// 'known' has a consistent meaning across all head maps in a stateless lexer
  /// (otherwise maybe only a subset of chars are known for a subset of actions,
  /// in this case the 'known' has an inconsistent meaning).
  /// This must be done before creating a head map because we need to iter over all known chars when filling the head map
  /// with [`HeadMatcher::Not`] and [`HeadMatcher::Unknown`].
  pub fn collect_all_known(
    actions: &Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  ) -> HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>> {
    let mut res = HashMap::new();

    for a in actions {
      if let Some(head_matcher) = a.head_matcher() {
        for c in match head_matcher {
          HeadMatcher::OneOf(set) => set,
          HeadMatcher::Not(set) => set,
          HeadMatcher::Unknown => continue,
        } {
          res.entry(*c).or_insert(Vec::new());
        }
      }
    }

    res
  }

  /// Create a self with a subset of actions and a known char map created by [`Self::collect_all_known`].
  pub fn new(
    actions: &Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
    known_map: HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
  ) -> Self {
    let mut res = Self {
      known_map,
      unknown_fallback: Vec::new(),
    };

    // fill the head map
    for a in actions {
      // no matter the action is muted or not, we need to check the head matcher
      // so we don't need to check if an action is muted or not here (like in literal map)
      // see [[@check if the action is muted or not in literal map]]

      if let Some(head_matcher) = a.head_matcher() {
        match head_matcher {
          HeadMatcher::OneOf(set) => {
            for c in set {
              res.known_map.get_mut(c).unwrap().push(a.clone());
            }
          }
          HeadMatcher::Not(set) => {
            for (c, vec) in res.known_map.iter_mut() {
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
        // no head matcher, add to all known chars
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

  pub fn known_map(&self) -> &HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>> {
    &self.known_map
  }
  pub fn unknown_fallback(&self) -> &Vec<Rc<Action<Kind, ActionState, ErrorType>>> {
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

  #[test]
  fn test_head_map() {
    let actions: Vec<Rc<Action<MockTokenKind<()>>>> = vec![
      exact("a"),
      exact("aa"),
      exact("b"),
      regex("[^c]").unchecked_head_not(['c']),
      regex(".").unchecked_head_unknown(),
      regex("a_muted").unchecked_head_in(['a']).mute(),
      regex("no_head_matcher").into(),
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

    // check action count
    assert_eq!(hm.known_map()[&'a'].len(), 5); // "a", "aa", "[^c]", a_muted, no_head_matcher
    assert_eq!(hm.known_map()[&'b'].len(), 3); // "b", "[^c]", no_head_matcher
    assert_eq!(hm.known_map()[&'c'].len(), 1); // no_head_matcher
    assert_eq!(hm.unknown_fallback().len(), 3); // "[^c]", ".", no_head_matcher
  }
}
