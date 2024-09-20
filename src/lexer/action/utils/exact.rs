use crate::{
  kind::MockKind,
  lexer::action::{eat_unchecked, simple_unchecked, Action},
};

pub use whitehole_helpers::{exact_vec, unchecked_exact_vec};

/// Match one string exactly, ***NO LOOKAHEAD***.
///
/// [`Action::head`] and [`Action::literal`] will be set automatically.
/// # Panics
/// Panics if the string is empty.
/// # Caveats
/// If there is only one char in the string,
/// it will be checked by head matcher, and this action will accept it directly.
/// This is OK when using with lexers, but might be faulty when being used standalone.
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
pub fn exact<'a, State, Heap>(s: impl Into<String>) -> Action<'a, MockKind<()>, State, Heap> {
  let s: String = s.into();
  let head = s.chars().next().expect("empty string is not allowed");
  let literal = s.clone();

  if s.len() == head.len_utf8() {
    // perf: if there is only one char, it will be checked by head matcher
    // so we can just eat it
    eat_unchecked(head.len_utf8())
  } else {
    simple_unchecked(move |input| {
      if input.rest().starts_with(&s) {
        s.len()
      } else {
        0
      }
    })
  }
  .unchecked_head_in([head])
  .unchecked_literal(literal)
}

/// Create an action for each char using [`exact`].
///
/// [`Action::head`] and [`Action::literal`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, exact_chars, exact};
/// # let actions: Vec<Action<_>> =
/// exact_chars("+-*/()");
/// // equals to
/// # let actions: Vec<Action<_>> =
/// vec![exact("+"), exact("-"), exact("*"), exact("/"), exact("("), exact(")")];
/// ```
#[inline]
pub fn exact_chars<'a, State, Heap>(s: &str) -> Vec<Action<'a, MockKind<()>, State, Heap>> {
  s.chars().map(|c| exact(c)).collect()
}

/// Match one string only by its first char instead of the whole string, ***NO LOOKAHEAD***.
///
/// [`Action::head`] and [`Action::literal`] will be set automatically.
/// # Panics
/// Panics if the string is empty.
/// # Caveats
/// You should only use this if you are sure the token is unique by its first char,
/// and the content you are lexing is valid to your format.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, unchecked_exact};
/// // this will only check if the first char is 't' by head matcher
/// // and digest 4 chars if it is
/// # let action: Action<_> =
/// unchecked_exact("true");
/// ```
pub fn unchecked_exact<'a, State, Heap>(
  s: impl Into<String>,
) -> Action<'a, MockKind<()>, State, Heap> {
  let s: String = s.into();
  let head = s.chars().next().expect("empty string is not allowed");
  let len = s.len();
  eat_unchecked(len)
    .unchecked_head_in([head])
    .unchecked_literal(s)
}

/// Create an action for each char using [`unchecked_exact`].
///
/// [`Action::head`] and [`Action::literal`] will be set automatically.
/// # Caveats
/// You should only use this if you are sure the token is unique by its first char,
/// and the content you are lexing is valid to your format.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, unchecked_exact_chars, unchecked_exact};
/// # let action: Vec<Action<_>> =
/// unchecked_exact_chars("+-*/");
/// // equals to
/// # let actions: Vec<Action<_>> =
/// vec![unchecked_exact("+"), unchecked_exact("-"), unchecked_exact("*"), unchecked_exact("/")];
/// ```
#[inline]
pub fn unchecked_exact_chars<'a, State, Heap>(
  s: impl Into<String>,
) -> Vec<Action<'a, MockKind<()>, State, Heap>> {
  // actually this is the same as `exact_chars` because the `exact` is optimized for one char.
  // however using `unchecked_exact` is more clear for the user,
  // and the building process is slightly more optimized since `unchecked_exact` is simpler than `exact`.
  s.into().chars().map(|c| unchecked_exact(c)).collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{ActionInput, HeadMatcher};
  use whitehole_helpers::_exact_vec;

  fn assert_accept(action: &Action<MockKind<()>>, text: &str, expected: usize) {
    assert_eq!(
      (action.exec.raw)(&mut ActionInput::new(text, 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .digested,
      expected
    );
  }
  fn assert_reject(action: &Action<MockKind<()>>, text: &str) {
    assert!((action.exec.raw)(&mut ActionInput::new(text, 0, &mut (), &mut ()).unwrap()).is_none());
  }

  #[should_panic]
  #[test]
  fn action_utils_exact_empty() {
    exact::<(), ()>("");
  }

  #[test]
  fn action_utils_exact() {
    let action: Action<MockKind<()>> = exact("aa");
    assert_reject(&action, "bb");
    assert_accept(&action, "aa", 2);
    // no lookahead
    assert_accept(&action, "aab", 2);
    // head matcher
    assert!(matches!(
      action.head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'a')
    ));
    // literal
    assert_eq!(action.literal(), &Some("aa".into()));
  }

  #[test]
  fn exact_one_char() {
    let action: Action<MockKind<()>> = exact("a");
    assert_accept(&action, "a", 1);
    // only check using head matcher, don't check the body
    assert_accept(&action, "b", 1);
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

  #[test]
  fn action_utils_unchecked_exact() {
    let action: Action<MockKind<()>> = unchecked_exact("ab");
    assert_accept(&action, "ab", 2);
    // no lookahead
    assert_accept(&action, "abb", 2);
    // head matcher
    assert!(matches!(
      action.head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'a')
    ));
    // literal
    assert_eq!(action.literal(), &Some("ab".into()));

    // only the first char is checked by head matcher, not the action exec
    assert_accept(&action, "aa", 2);
    assert_accept(&action, "bb", 2);
  }

  #[test]
  #[should_panic]
  fn action_utils_unchecked_exact_empty() {
    unchecked_exact::<(), ()>("");
  }

  #[test]
  fn action_utils_unchecked_exact_chars() {
    let actions: Vec<Action<_>> = unchecked_exact_chars("+-*/");
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

    // only the first char is checked by head matcher, not the action exec
    assert_accept(&actions[0], "1", 1);
    assert_accept(&actions[1], "1", 1);
    assert_accept(&actions[2], "1", 1);
    assert_accept(&actions[3], "1", 1);
  }
}
