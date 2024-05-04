use crate::lexer::action::Action;
use std::{collections::HashMap, rc::Rc};

pub(super) struct LiteralMap<Kind: 'static, ActionState, ErrorType> {
  /// Store actions for known literals.
  known_map: HashMap<String, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
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

  /// Create a self with a subset of actions and a known literal map created by [`Self::collect_all_known`].
  pub fn new(
    actions: &Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
    known_map: HashMap<String, Vec<Rc<Action<Kind, ActionState, ErrorType>>>>,
  ) -> Self {
    let mut res = Self { known_map };

    // fill the action map
    for a in actions {
      if a.muted() {
        // muted, expectation.literal will be ignored, add to all known literals
        for vec in res.known_map.values_mut() {
          vec.push(a.clone());
        }
        // ignore self.literal, just continue
        continue;
      }

      if let Some(literal) = a.literal() {
        res.known_map.get_mut(literal).unwrap().push(a.clone());
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
    action::{exact, regex},
    token::MockTokenKind,
  };

  #[test]
  fn test_literal_map() {
    let actions: Vec<Rc<Action<MockTokenKind<()>>>> = vec![
      exact("a"),        // has literal, not muted
      exact("a").mute(), // has literal, muted
      exact("aa"),
      regex("no_literal").into(),
      regex("muted_no_literal").into_action().mute(),
    ]
    .into_iter()
    .map(Rc::new)
    .collect();

    let lm = LiteralMap::new(&actions, LiteralMap::collect_all_known(&actions));

    // collect all literals
    assert!(lm.known_map().contains_key("a"));
    assert!(lm.known_map().contains_key("aa"));
    assert_eq!(lm.known_map().len(), 2);

    // muted actions are added to all known literals
    assert_eq!(lm.known_map().get("a").unwrap().len(), 3); // "a", muted "a", muted_no_literal
    assert_eq!(lm.known_map().get("aa").unwrap().len(), 3); // muted "a", "aa", muted_no_literal
  }
}
