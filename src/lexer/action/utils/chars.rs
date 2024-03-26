use crate::lexer::{
  action::{simple, Action},
  token::MockTokenKind,
};
use std::{collections::HashSet, ops::RangeInclusive};

/// Match chars greedily by a condition.
/// If no chars are matched, reject.
///
/// It's recommended to set [`Action::head_matcher`] to optimize the lex performance.
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
///
/// The [`Action::head_matcher`] will be set automatically.
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
  chars(move |ch| range.contains(ch)).unchecked_head_in([head])
}

/// Match chars greedily by a set.
/// If no chars are matched, reject.
///
/// The [`Action::head_matcher`] will be set automatically.
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
  chars(move |ch| charset.contains(ch)).unchecked_head_in(head)
}

/// Match unicode whitespaces greedy.
/// For the list of whitespaces, see https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt.
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::whitespaces;
/// # use whitehole::lexer::LexerBuilder;
/// # use whitehole_macros::TokenKind;
/// # #[derive(TokenKind, Default, Clone)]
/// # enum MyKind { #[default] Anonymous }
/// # let builder = LexerBuilder::<MyKind>::new();
/// builder.ignore_default(whitespaces());
/// ```
pub fn whitespaces<ActionState, ErrorType>() -> Action<MockTokenKind<()>, ActionState, ErrorType> {
  // TODO: benchmark this vs regex `^\s+`
  chars(|ch| ch.is_whitespace())
    // 0009..000D    ; White_Space # Cc   [5] <control-0009>..<control-000D>
    // 0020          ; White_Space # Zs       SPACE
    // 0085          ; White_Space # Cc       <control-0085>
    // 00A0          ; White_Space # Zs       NO-BREAK SPACE
    // 1680          ; White_Space # Zs       OGHAM SPACE MARK
    // 2000..200A    ; White_Space # Zs  [11] EN QUAD..HAIR SPACE
    // 2028          ; White_Space # Zl       LINE SEPARATOR
    // 2029          ; White_Space # Zp       PARAGRAPH SEPARATOR
    // 202F          ; White_Space # Zs       NARROW NO-BREAK SPACE
    // 205F          ; White_Space # Zs       MEDIUM MATHEMATICAL SPACE
    // 3000          ; White_Space # Zs       IDEOGRAPHIC SPACE
    .unchecked_head_in([
      '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{0020}', '\u{0085}',
      '\u{00A0}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}',
      '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{2028}',
      '\u{2029}', '\u{202F}', '\u{205F}', '\u{3000}',
    ])
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{ActionInput, HeadMatcher};

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

  #[test]
  fn action_utils_whitespaces() {
    let action: Action<MockTokenKind<()>> = whitespaces();

    // common cases
    assert_reject(&action, "123");
    assert_reject(&action, "abc");
    assert_accept(&action, " \n\t", 3);

    // full cases
    let text: String = [
      '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{0020}', '\u{0085}',
      '\u{00A0}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}',
      '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{2028}',
      '\u{2029}', '\u{202F}', '\u{205F}', '\u{3000}',
    ]
    .into_iter()
    .collect();
    assert_accept(&action, &text, text.len());

    // head matcher
    assert!(matches!(
      action.head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == text.chars().count() && set.iter().all(|c| text.contains(*c))
    ));
  }
}
