use super::StringList;
use crate::lexer::{
  action::{simple, Action},
  token::MockTokenKind,
};

/// Match one string exactly, ***NO LOOKAHEAD***.
///
/// [`Action::head_matcher`] and [`Action::literal`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, exact};
/// # let action: Action<_> =
/// exact("true"); // with a &str
/// # let action: Action<_> =
/// exact(';'); // with a char
/// ```
pub fn exact<ActionState, ErrorType>(
  s: impl Into<String>,
) -> Action<MockTokenKind<()>, ActionState, ErrorType> {
  let s: String = s.into();
  let head = s.chars().next().unwrap(); // TODO: add comment for the panic
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

/// Create an action for each string using [`exact`].
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, exact_vec, exact};
/// # let actions: Vec<Action<_>> =
/// exact_vec(["++", "--"]);
/// // equals to
/// vec![exact("++"), exact("--")];
/// ```
// TODO: make this a macro
pub fn exact_vec<ActionState, ErrorType>(
  ss: impl Into<StringList>,
) -> Vec<Action<MockTokenKind<()>, ActionState, ErrorType>> {
  ss.into().0.into_iter().map(|s| exact(s)).collect()
}

/// Create an action for each char using [`exact`].
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, exact_chars};
/// # let actions: Vec<Action<_>> =
/// exact_chars("+-*/()");
/// // equals to
/// vec![exact("+"), exact("-"), exact("*"), exact("/"), exact("("), exact(")")];
/// ```
pub fn exact_chars<ActionState, ErrorType>(
  s: impl Into<String>,
) -> Vec<Action<MockTokenKind<()>, ActionState, ErrorType>> {
  s.into().chars().map(|c| exact(c.to_string())).collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{ActionInput, HeadMatcher};

  fn assert_accept(action: &Action<MockTokenKind<()>>, text: &str, expected: usize) {
    assert_eq!(
      action
        .exec(&mut ActionInput::new(text, 0, &mut ()).unwrap())
        .unwrap()
        .digested,
      expected
    );
  }
  fn assert_reject(action: &Action<MockTokenKind<()>>, text: &str) {
    assert!(action
      .exec(&mut ActionInput::new(text, 0, &mut ()).unwrap())
      .is_none());
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
      action.head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'a')
    ));
    // literal
    assert_eq!(action.literal(), &Some("a".into()));
  }

  #[test]
  fn action_utils_exact_vec() {
    let actions: Vec<Action<MockTokenKind<()>>> = exact_vec(["++", "--"]);
    assert_accept(&actions[0], "++", 2);
    assert_accept(&actions[1], "--", 2);
    // no lookahead
    assert_accept(&actions[0], "+++", 2);
    assert_accept(&actions[1], "---", 2);
    // head matcher
    assert!(matches!(
      actions[0].head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'+')
    ));
    assert!(matches!(
      actions[1].head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'-')
    ));
  }

  #[test]
  fn action_utils_exact_chars() {
    let actions: Vec<Action<MockTokenKind<()>>> = exact_chars("+-*/");
    assert_accept(&actions[0], "+", 1);
    assert_accept(&actions[1], "-", 1);
    assert_accept(&actions[2], "*", 1);
    assert_accept(&actions[3], "/", 1);
    // no lookahead
    assert_accept(&actions[0], "++", 1);
    assert_accept(&actions[1], "--", 1);
    // head matcher
    assert!(matches!(
      actions[0].head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'+')
    ));
    assert!(matches!(
      actions[1].head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'-')
    ));
    assert!(matches!(
      actions[2].head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'*')
    ));
    assert!(matches!(
      actions[3].head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'/')
    ));
  }
}
