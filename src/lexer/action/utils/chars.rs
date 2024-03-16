use crate::lexer::{action::simple, token::MockTokenKind, Action};
use std::{collections::HashSet, ops::RangeInclusive};

/// Match chars greedily by a condition.
/// If no chars are matched, reject.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, chars};
/// # let action: Action<()> =
/// chars(|ch| ch.is_ascii_digit());
/// ```
pub fn chars<ActionState, ErrorType, F>(
  condition: F,
) -> Action<MockTokenKind<()>, ActionState, ErrorType>
where
  F: Fn(&char) -> bool + 'static,
{
  simple(move |input| {
    let mut i = 0;
    // TODO: maybe someday we can get a `&char` instead of a `char` here
    for ch in input.rest().chars() {
      if !condition(&ch) {
        break;
      }
      i += ch.len_utf8();
    }
    i
  })
}

/// Match chars greedily by a range.
/// If no chars are matched, reject.
/// The head matcher will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, char_range};
/// # let action: Action<()> =
/// char_range('0'..='9');
/// ```
pub fn char_range<ActionState, ErrorType>(
  range: impl Into<RangeInclusive<char>>,
) -> Action<MockTokenKind<()>, ActionState, ErrorType> {
  let range: RangeInclusive<_> = range.into();
  let head = *range.start();
  chars(move |ch| range.contains(ch)).head_in([head])
}

/// Match chars greedily by a set.
/// If no chars are matched, reject.
/// The head matcher will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, charset};
/// # let action: Action<()> =
/// charset(['a', 'b', 'c']);
/// ```
pub fn charset<ActionState, ErrorType>(
  set: impl Into<HashSet<char>>,
) -> Action<MockTokenKind<()>, ActionState, ErrorType> {
  let charset: HashSet<_> = set.into();
  let head = charset.clone();
  chars(move |ch| charset.contains(ch)).head_in(head)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::ActionInput;

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
  fn action_utils_chars() {
    let action = chars(|ch| ch.is_ascii_digit());
    assert_reject(&action, "abc");
    assert_accept(&action, "123", 3);
    assert_accept(&action, "123abc", 3);
  }

  #[test]
  fn action_utils_char_range() {
    let action = char_range('0'..='9');
    assert_reject(&action, "abc");
    assert_accept(&action, "123", 3);
    assert_accept(&action, "123abc", 3);
  }

  #[test]
  fn action_utils_charset() {
    let action = charset(['a', 'b', 'c']);
    assert_reject(&action, "123");
    assert_accept(&action, "abc", 3);
    assert_accept(&action, "abc123", 3);
  }
}
