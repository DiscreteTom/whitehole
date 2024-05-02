use crate::lexer::action::{Action, HeadMatcher};
use std::{collections::HashMap, rc::Rc};

pub(crate) struct HeadMap<Kind: 'static, ActionState, ErrorType> {
  /// Store actions for known chars.
  known_map: HashMap<char, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
  /// Store actions for unknown chars.
  unknown_fallback: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
}

impl<Kind, ActionState, ErrorType> HeadMap<Kind, ActionState, ErrorType> {
  pub fn new(actions: &Vec<Rc<Action<Kind, ActionState, ErrorType>>>) -> Self {
    let mut res = Self {
      known_map: HashMap::new(),
      unknown_fallback: Vec::new(),
    };
    // collect all known chars, this must be done before filling the head map
    // because we need to iter over all known chars when filling the head map
    for a in actions {
      if let Some(head_matcher) = a.head_matcher() {
        for c in match head_matcher {
          HeadMatcher::OneOf(set) => set,
          HeadMatcher::Not(set) => set,
          HeadMatcher::Unknown => continue,
        } {
          res.known_map.entry(*c).or_insert(Vec::new());
        }
      }
    }
    // fill the head map
    for a in actions {
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
