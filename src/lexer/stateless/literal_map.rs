use crate::lexer::action::Action;
use std::{collections::HashMap, rc::Rc};

pub(crate) struct LiteralMap<Kind: 'static, ActionState, ErrorType> {
  /// Store actions for known literals.
  known_map: HashMap<String, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
  // for literal map there is no unknown_fallback because we don't check
  // actions with mismatched literals
}

impl<Kind, ActionState, ErrorType> LiteralMap<Kind, ActionState, ErrorType> {
  pub fn new(actions: &Vec<Rc<Action<Kind, ActionState, ErrorType>>>) -> Self {
    let mut res = Self {
      known_map: HashMap::new(),
    };
    // collect all known literals, this must be done before filling the action map
    // because we need to iter over all known literals when filling the action map
    for a in actions {
      if let Some(literal) = a.literal() {
        res.known_map.entry(literal.clone()).or_insert(Vec::new());
      }
    }
    // fill the action map
    for a in actions {
      if let Some(literal) = a.literal() {
        res.known_map.get_mut(literal).unwrap().push(a.clone());
      } else {
        // no literal. if muted, add to all known literals
        if a.muted() {
          for vec in res.known_map.values_mut() {
            vec.push(a.clone());
          }
        }
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    res
  }

  pub fn known_map(&self) -> &HashMap<String, Vec<Rc<Action<Kind, ActionState, ErrorType>>>> {
    &self.known_map
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::{exact, whitespaces},
    token::MockTokenKind,
  };

  #[test]
  fn test_literal_map() {
    let lm: LiteralMap<MockTokenKind<()>, (), ()> = LiteralMap::new(
      &vec![exact("a"), exact("a"), exact("aa"), whitespaces().mute()]
        .into_iter()
        .map(Rc::new)
        .collect(),
    );

    // collect all literals
    assert!(lm.known_map.contains_key("a"));
    assert!(lm.known_map.contains_key("aa"));
    assert_eq!(lm.known_map.len(), 2);

    // muted actions are added to all known literals
    assert_eq!(lm.known_map.get("a").unwrap().len(), 3);
    assert_eq!(lm.known_map.get("aa").unwrap().len(), 2);
  }
}
