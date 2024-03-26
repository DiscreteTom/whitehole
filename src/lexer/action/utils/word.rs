use super::StringList;
use crate::lexer::{action::simple, token::MockTokenKind, Action};
use std::collections::HashSet;

/// Match one of the provided words, in one action,
/// ***LOOKAHEAD*** one char to ensure there is a word boundary
/// (alphanumeric or `_`) or end of input after the word.
/// Stop at the first match.
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, word};
/// # let action: Action<MockTokenKind<()>> =
/// // single word
/// word("a");
/// # let action: Action<MockTokenKind<()>> =
/// // multiple words
/// // try to match "a" first, then "b", in one action
/// word(["a", "b"]);
/// ```
/// # Caveats
/// Be ware if you provide multiple words:
/// ```
/// # use whitehole::lexer::action::{Action, word};
/// // this will skip the check of `"a"` when re-lex
/// // since this is one action instead of two.
/// # let action: Action<MockTokenKind<()>> =
/// word(["ab", "a"]);
/// ```
/// To avoid the above, try [`word_vec`] or [`word_chars`].
pub fn word<ActionState, ErrorType>(
  ss: impl Into<StringList>,
) -> Action<MockTokenKind<()>, ActionState, ErrorType> {
  // don't use `exact(ss).reject_if(...)` here
  // e.g. `exact(["a", "ab"])` will accept "ab" as "a"
  // then reject since no word boundary after "a"
  // however "ab" is accepted by `word(["a", "ab"])`

  let ss: Vec<String> = ss.into().0;

  if ss.len() == 0 {
    panic!("empty word list");
  }

  // optimize for single string // TODO: is this needed?
  if ss.len() == 1 {
    let s = ss.into_iter().next().unwrap();
    let head = s.chars().next().unwrap();
    return simple(move |input| {
      if input.rest().starts_with(&s)
        && input.rest()[s.len()..]
          .chars()
          .next()
          // if next char exists, it can't be alphanumeric or `_`
          .map(|c| !c.is_alphanumeric() && c != '_')
          // if no next char (EOF), it's ok
          .unwrap_or(true)
      {
        s.len()
      } else {
        0
      }
    })
    .unchecked_head_in([head]);
  }

  let heads: HashSet<_> = ss.iter().map(|s| s.chars().next().unwrap()).collect();
  simple(move |input| {
    for s in &ss {
      if input.rest().starts_with(s)
        && input.rest()[s.len()..]
          .chars()
          .next()
          // if next char exists, it can't be alphanumeric or `_`
          .map(|c| !c.is_alphanumeric() && c != '_')
          // if no next char (EOF), it's ok
          .unwrap_or(true)
      {
        return s.len();
      }
    }
    0 // no match
  })
  .unchecked_head_in(heads)
}

/// Similar to [`word`], but create an action for each string.
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, word_vec};
/// # let actions: Vec<Action<MockTokenKind<()>>> =
/// word_vec(["int", "bool"]);
/// ```
pub fn word_vec<ActionState, ErrorType>(
  ss: impl Into<StringList>,
) -> Vec<Action<MockTokenKind<()>, ActionState, ErrorType>> {
  let ss: Vec<String> = ss.into().0;

  if ss.len() == 0 {
    panic!("empty word list");
  }

  ss.into_iter().map(|s| word(s)).collect()
}

/// Similar to [`word`], but accept one string
/// and create an action for each char.
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, word_chars};
/// # let actions: Vec<Action<MockTokenKind<()>>> =
/// word_chars("abc");
/// ```
pub fn word_chars<ActionState, ErrorType>(
  s: impl Into<String>,
) -> Vec<Action<MockTokenKind<()>, ActionState, ErrorType>> {
  let s: String = s.into();

  if s.len() == 0 {
    panic!("empty string");
  }

  s.chars().map(|c| word(c.to_string())).collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{ActionInput, HeadMatcher};
  #[test]
  #[should_panic]
  fn action_utils_word_empty() {
    word::<(), ()>(vec![]);
  }

  fn assert_accept(action: &Action<MockTokenKind<()>>, text: &str, expected: usize) {
    assert_eq!(
      action
        .exec(&mut ActionInput::new(text, 0, &mut ()))
        .unwrap()
        .digested,
      expected
    );
  }
  fn assert_reject(action: &Action<MockTokenKind<()>>, text: &str) {
    assert!(action
      .exec(&mut ActionInput::new(text, 0, &mut ()))
      .is_none());
  }

  #[test]
  fn action_utils_word() {
    // single string
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

    // multi strings
    let action: Action<MockTokenKind<()>> = word(["a", "b"]);
    assert_reject(&action, "c");
    assert_reject(&action, "aa");
    assert_reject(&action, "bc");
    assert_accept(&action, "a", 1);
    assert_accept(&action, "a ", 1);
    assert_accept(&action, "b", 1);
    assert_accept(&action, "b,", 1);
    // head matcher
    assert!(matches!(
      action.head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 2 && set.contains(&'a') && set.contains(&'b')
    ));

    // caveats
    // this will digest 1 by `exact` but digest 2 by `word`
    let action: Action<MockTokenKind<()>> = word(["a", "ab"]);
    assert_accept(&action, "ab", 2);
  }

  #[test]
  #[should_panic]
  fn action_utils_word_vec_empty() {
    word_vec::<(), ()>(vec![]);
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

  #[test]
  #[should_panic]
  fn action_utils_word_chars_empty() {
    word_chars::<(), ()>("");
  }

  #[test]
  fn action_utils_word_chars() {
    let actions: Vec<Action<MockTokenKind<()>>> = word_chars("abc");
    assert_accept(&actions[0], "a", 1);
    assert_accept(&actions[1], "b", 1);
    assert_accept(&actions[2], "c", 1);
    // lookahead
    assert_accept(&actions[0], "a ", 1);
    assert_accept(&actions[1], "b\t", 1);
    assert_accept(&actions[2], "c\n", 1);
    assert_reject(&actions[0], "ab");
    assert_reject(&actions[1], "bc");
    assert_reject(&actions[2], "ca");
    // head matcher
    assert!(matches!(
      actions[0].head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'a')
    ));
    assert!(matches!(
      actions[1].head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'b')
    ));
    assert!(matches!(
      actions[2].head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'c')
    ));
  }
}
