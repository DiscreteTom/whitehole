use super::StringList;
use crate::lexer::{action::simple, Action};
use std::collections::HashSet;

/// Match one of the provided strings exactly, in one action, ***NO LOOKAHEAD***.
/// Stop at the first match.
/// The head matcher will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, exact};
/// # let action: Action<()> =
/// // single string
/// exact("a");
/// # let action: Action<()> =
/// // multiple strings
/// // try to match "a" first, then "b", in one action
/// exact(["a", "b"]);
/// ```
/// # Caveats
/// Be ware if you provide multiple strings:
/// ```
/// # use whitehole::lexer::action::{Action, exact};
/// // this will always match `"a"` and never match `"ab"`
/// # let action: Action<()> =
/// exact(["a", "ab"]);
/// // this will skip the check of `"a"` when re-lex
/// // since this is one action instead of two.
/// # let action: Action<()> =
/// exact(["ab", "a"]);
/// ```
/// To avoid the above, try [`exact_vec`] or [`exact_chars`].
pub fn exact<ActionState, ErrorType>(
  ss: impl Into<StringList>,
) -> Action<(), ActionState, ErrorType> {
  let ss: Vec<String> = ss.into().0;

  if ss.len() == 0 {
    panic!("empty string list");
  }

  // optimize for single string // TODO: is this needed?
  if ss.len() == 1 {
    let s = ss.into_iter().next().unwrap();
    let head = s.chars().next().unwrap();
    return simple(move |input| {
      if input.rest().starts_with(&s) {
        s.len()
      } else {
        0
      }
    })
    .head_in([head]);
  }

  let heads: HashSet<_> = ss.iter().map(|s| s.chars().next().unwrap()).collect();
  simple(move |input| {
    for s in &ss {
      if input.rest().starts_with(s) {
        return s.len();
      }
    }
    0 // no match
  })
  .head_in(heads)
}

/// Similar to [`exact`], but create an action for each string.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, exact_vec};
/// # let actions: Vec<Action<()>> =
/// exact_vec(["++", "--"]);
/// ```
pub fn exact_vec<ActionState, ErrorType>(
  ss: impl Into<StringList>,
) -> Vec<Action<(), ActionState, ErrorType>> {
  let ss: Vec<String> = ss.into().0;

  if ss.len() == 0 {
    panic!("empty string list");
  }

  ss.into_iter().map(|s| exact(s)).collect()
}

/// Similar to [`exact`], but accept one string
/// and create an action for each char.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, exact_chars};
/// # let actions: Vec<Action<()>> =
/// exact_chars("+-*/()");
/// ```
pub fn exact_chars<ActionState, ErrorType>(
  s: impl Into<String>,
) -> Vec<Action<(), ActionState, ErrorType>> {
  let s: String = s.into();

  if s.len() == 0 {
    panic!("empty string");
  }

  s.chars().map(|c| exact(c.to_string())).collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{ActionInput, ActionInputRestHeadMatcher};

  #[test]
  #[should_panic]
  fn action_utils_exact_empty() {
    exact::<(), ()>(vec![]);
  }

  #[test]
  fn action_utils_exact() {
    // single string
    let action: Action<()> = exact("a");
    let text = "a";
    assert_eq!(
      action
        .exec(&mut ActionInput::new(text, 0, &mut ()))
        .unwrap()
        .digested,
      text.len()
    );
    // head matcher
    assert!(matches!(
      action.head_matcher().as_ref().unwrap(),
      ActionInputRestHeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'a')
    ));
    // no lookahead
    let text = "ab";
    assert_eq!(
      action
        .exec(&mut ActionInput::new(text, 0, &mut ()))
        .unwrap()
        .digested,
      1
    );

    // multi strings
    let action: Action<()> = exact(["a", "b"]);
    let text = "b";
    assert_eq!(
      action
        .exec(&mut ActionInput::new(text, 0, &mut ()))
        .unwrap()
        .digested,
      text.len()
    );
    // head matcher
    assert!(matches!(
      action.head_matcher().as_ref().unwrap(),
      ActionInputRestHeadMatcher::OneOf(set) if set.len() == 2 && set.contains(&'a') && set.contains(&'b')
    ));

    // caveats
    let action: Action<()> = exact(["a", "ab"]);
    let text = "ab";
    assert_eq!(
      action
        .exec(&mut ActionInput::new(text, 0, &mut ()))
        .unwrap()
        .digested,
      1
    );
  }
}
