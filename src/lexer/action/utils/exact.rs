use crate::lexer::{
  action::{simple, Action},
  token::MockTokenKind,
};

pub use whitehole_helpers::exact_vec;

/// Match one string exactly, ***NO LOOKAHEAD***.
///
/// [`Action::head`] and [`Action::literal`] will be set automatically.
/// # Panics
/// Panics if the string is empty.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, exact};
/// # let action: Action<_> =
/// exact("true".to_string()); // using String
/// # let action: Action<_> =
/// exact("true"); // using &str
/// # let action: Action<_> =
/// exact(';'); // using char
/// ```
pub fn exact<State, ErrorType>(
  s: impl Into<String>,
) -> Action<MockTokenKind<()>, State, ErrorType> {
  let s: String = s.into();
  let head = s.chars().next().expect("empty string is not allowed");
  let literal = Some(s.clone());
  let mut a = simple(move |input| {
    if input.rest().starts_with(&s) {
      s.len()
    } else {
      0
    }
  })
  .unchecked_head_in([head]);
  a.literal = literal;
  a
}

/// Create an action for each char using [`exact`].
///
/// The [`Action::head`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, exact_chars, exact};
/// # let actions: Vec<Action<_>> =
/// exact_chars("+-*/()");
/// // equals to
/// # let actions: Vec<Action<_>> =
/// vec![exact("+"), exact("-"), exact("*"), exact("/"), exact("("), exact(")")];
/// ```
pub fn exact_chars<State, ErrorType>(
  s: impl Into<String>,
) -> Vec<Action<MockTokenKind<()>, State, ErrorType>> {
  s.into().chars().map(|c| exact(c)).collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{ActionExec, ActionInput, HeadMatcher};
  use whitehole_helpers::_exact_vec;

  fn assert_accept(action: &Action<MockTokenKind<()>>, text: &str, expected: usize) {
    assert_eq!(
      match &action.exec {
        ActionExec::Immutable(exec) =>
          exec(&mut ActionInput::new(text, 0, &()).unwrap())
            .unwrap()
            .digested,
        _ => unreachable!(),
      },
      expected
    );
  }
  fn assert_reject(action: &Action<MockTokenKind<()>>, text: &str) {
    assert!(action.exec.as_immutable()(&ActionInput::new(text, 0, &()).unwrap()).is_none());
  }

  #[should_panic]
  #[test]
  fn action_utils_exact_empty() {
    exact::<(), ()>("");
  }

  #[test]
  fn action_utils_exact() {
    let action: Action<MockTokenKind<()>> = exact("a");
    assert_reject(&action, "b");
    assert_accept(&action, "a", 1);
    // no lookahead
    assert_accept(&action, "ab", 1);
    // head matcher
    assert!(matches!(
      action.head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'a')
    ));
    // literal
    assert_eq!(action.literal(), &Some("a".into()));
  }

  #[test]
  fn action_utils_exact_vec() {
    let actions: Vec<Action<_>> = _exact_vec!["++", "--"];
    assert_accept(&actions[0], "++", 2);
    assert_accept(&actions[1], "--", 2);
    // no lookahead
    assert_accept(&actions[0], "+++", 2);
    assert_accept(&actions[1], "---", 2);
    // head matcher
    assert!(matches!(
      actions[0].head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'+')
    ));
    assert!(matches!(
      actions[1].head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'-')
    ));
  }

  #[should_panic]
  #[test]
  fn action_utils_exact_vec_macro_empty() {
    let _: Vec<Action<_>> = _exact_vec![""];
  }

  #[test]
  fn action_utils_exact_chars() {
    let actions: Vec<Action<_>> = exact_chars("+-*/");
    assert_accept(&actions[0], "+", 1);
    assert_accept(&actions[1], "-", 1);
    assert_accept(&actions[2], "*", 1);
    assert_accept(&actions[3], "/", 1);
    // no lookahead
    assert_accept(&actions[0], "++", 1);
    assert_accept(&actions[1], "--", 1);
    // head matcher
    assert!(matches!(
      actions[0].head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'+')
    ));
    assert!(matches!(
      actions[1].head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'-')
    ));
    assert!(matches!(
      actions[2].head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'*')
    ));
    assert!(matches!(
      actions[3].head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'/')
    ));
  }
}
