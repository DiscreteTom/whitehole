use super::exact;
use crate::lexer::{
  action::{AcceptedActionOutputContext, Action, ActionInput, ActionOutput},
  token::MockTokenKind,
};

pub use whitehole_helpers::word_vec;

/// Return `true` if the first char of `rest` is a word boundary (non-alphanumeric and not `_`),
/// or `rest` is empty.
/// # Examples
/// ```
/// # use whitehole::lexer::action::starts_with_word_boundary;
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
#[inline]
pub fn starts_with_word_boundary(rest: &str) -> bool {
  rest
    .chars()
    .next()
    // if next char exists, it can't be alphanumeric or `_`
    .map(|c| !c.is_alphanumeric() && c != '_')
    // if no next char (reach EOF), it's ok
    .unwrap_or(true)
}

/// Return `true` if there is no word boundary at the beginning of
/// [`ctx.rest`](AcceptedActionOutputContext::rest).
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, exact, no_word_boundary_in_rest};
/// # let _: Action<_> =
/// exact("hello").reject_if(no_word_boundary_in_rest);
/// ```
#[inline]
pub fn no_word_boundary_in_rest<State, Heap>(
  ctx: AcceptedActionOutputContext<
    &mut ActionInput<&mut State, &mut Heap>,
    &ActionOutput<MockTokenKind<()>>,
  >,
) -> bool {
  !starts_with_word_boundary(ctx.rest())
}

/// Match one word,
/// ***LOOKAHEAD*** one char to ensure there is a word boundary
/// (non-alphanumeric and not `_`) or end of input after the word.
///
/// The [`Action::head`] and [`Action::literal`] will be set automatically.
/// # Panics
/// Panics if the string is empty.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, word};
/// # let action: Action<_> =
/// word("import");
/// ```
#[inline]
pub fn word<State: 'static, Heap: 'static>(
  s: impl Into<String>,
) -> Action<MockTokenKind<()>, State, Heap> {
  exact(s).reject_if(no_word_boundary_in_rest)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{ActionInput, HeadMatcher};
  use whitehole_helpers::_word_vec;

  fn assert_accept(action: &Action<MockTokenKind<()>>, text: &str, expected: usize) {
    assert_eq!(
      (action.exec.raw)(&mut ActionInput::new(text, 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .digested,
      expected
    );
  }
  fn assert_reject(action: &Action<MockTokenKind<()>>, text: &str) {
    assert!((action.exec.raw)(&mut ActionInput::new(text, 0, &mut (), &mut ()).unwrap()).is_none());
  }

  #[should_panic]
  #[test]
  fn action_utils_word_empty() {
    word::<(), ()>("");
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
      action.head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'a')
    ));
    // literal
    assert_eq!(action.literal(), &Some("a".into()));
  }

  #[test]
  fn action_utils_word_vec() {
    let actions: Vec<Action<MockTokenKind<()>>> = _word_vec!["int", "bool"];
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
      actions[0].head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'i')
    ));
    assert!(matches!(
      actions[1].head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'b')
    ));
  }

  #[should_panic]
  #[test]
  fn action_utils_word_vec_empty() {
    let _: Vec<Action<_>> = _word_vec![""];
  }
}
