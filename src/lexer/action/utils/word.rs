use super::{exact, StringList};
use crate::lexer::{
  action::{AcceptedActionOutputContext, Action, ActionInput, ActionOutput},
  token::MockTokenKind,
};

/// Return `true` if the first char of `rest` is a word boundary (non-alphanumeric and not `_`),
/// or `rest` is empty.
/// # Examples
/// ```
/// # use whitehole::lexer::action::utils::word::starts_with_word_boundary;
/// assert!(starts_with_word_boundary(""));
/// assert!(starts_with_word_boundary(" "));
/// assert!(starts_with_word_boundary("\n"));
/// assert!(starts_with_word_boundary("\t"));
/// assert!(starts_with_word_boundary("+"));
/// assert!(starts_with_word_boundary(","));
/// assert!(starts_with_word_boundary("="));
/// assert!(starts_with_word_boundary("，"));
/// assert!(!starts_with_word_boundary("a"));
/// assert!(!starts_with_word_boundary("1"));
/// assert!(!starts_with_word_boundary("_"));
/// assert!(!starts_with_word_boundary("我"));
/// ```
pub fn starts_with_word_boundary(rest: &str) -> bool {
  rest
    .chars()
    .next()
    // if next char exists, it can't be alphanumeric or `_`
    .map(|c| !c.is_alphanumeric() && c != '_')
    // if no next char (EOF), it's ok
    .unwrap_or(true)
}

/// Return `true` if there is no word boundary at the beginning of
/// [`ctx.rest`](AcceptedActionOutputContext::rest).
/// # Examples
/// ```
/// # use whitehole::lexer::action::utils::word::no_word_boundary_in_rest;
/// # use whitehole::lexer::action::exact;
/// # let _: Action<_> =
/// exact("hello").reject_if(no_word_boundary_in_rest);
/// ```
pub fn no_word_boundary_in_rest<ActionState, ErrorType>(
  ctx: AcceptedActionOutputContext<
    &ActionInput<ActionState>,
    &ActionOutput<MockTokenKind<()>, Option<ErrorType>>,
  >,
) -> bool {
  !starts_with_word_boundary(ctx.rest())
}

/// Match one word,
/// ***LOOKAHEAD*** one char to ensure there is a word boundary
/// (non-alphanumeric and not `_`) or end of input after the word.
///
/// The [`Action::head_matcher`] and [`Action::literal`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, word};
/// # let action: Action<_> =
/// word("import");
/// ```
pub fn word<ActionState: 'static, ErrorType: 'static>(
  s: impl Into<String>,
) -> Action<MockTokenKind<()>, ActionState, ErrorType> {
  exact(s).reject_if(no_word_boundary_in_rest)
}

/// Create an action for each string using [`word`].
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, word_vec};
/// # let actions: Vec<Action<MockTokenKind<()>>> =
/// word_vec(["int", "bool"]);
/// ```
pub fn word_vec<ActionState: 'static, ErrorType: 'static>(
  ss: impl Into<StringList>,
) -> Vec<Action<MockTokenKind<()>, ActionState, ErrorType>> {
  ss.into().0.into_iter().map(|s| word(s)).collect()
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
  fn action_utils_word() {
    let action: Action<MockTokenKind<()>> = word("a");
    assert_reject(&action, "b");
    assert_accept(&action, "a", 1);
    // lookahead
    assert_accept(&action, "a ", 1);
    assert_accept(&action, "a\t", 1);
    assert_accept(&action, "a\n", 1);
    assert_accept(&action, "a+", 1);
    assert_accept(&action, "a,", 1);
    assert_accept(&action, "a=", 1);
    assert_accept(&action, "a，", 1); // punctuation in other languages also count as word boundary
    assert_reject(&action, "ab");
    assert_reject(&action, "a1");
    assert_reject(&action, "a_");
    assert_reject(&action, "a我");
    // head matcher
    assert!(matches!(
      action.head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'a')
    ));
  }

  #[test]
  fn action_utils_word_vec() {
    let actions: Vec<Action<MockTokenKind<()>>> = word_vec(["int", "bool"]);
    assert_accept(&actions[0], "int", 3);
    assert_accept(&actions[1], "bool", 4);
    // lookahead
    assert_accept(&actions[0], "int ", 3);
    assert_accept(&actions[1], "bool\t", 4);
    assert_accept(&actions[0], "int+", 3);
    assert_accept(&actions[1], "bool,", 4);
    assert_reject(&actions[0], "int1");
    assert_reject(&actions[1], "bool_");
    // head matcher
    assert!(matches!(
      actions[0].head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'i')
    ));
    assert!(matches!(
      actions[1].head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'b')
    ));
  }
}
