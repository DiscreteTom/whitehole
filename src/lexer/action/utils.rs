mod exact;
mod float;
mod integer;
pub mod json;
mod regex;
mod string;
mod word;

pub use exact::*;
pub use float::*;
pub use integer::*;
pub use regex::*;
pub use string::*;
pub use word::*;

use super::{simple::simple, Action};
use crate::lexer::token::MockTokenKind;
use std::{collections::HashSet, ops::RangeInclusive};

/// Match chars by the condition greedily.
///
/// It's recommended to set [`Action::head`] to optimize the lex performance.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, chars};
/// // match all ascii digits greedily
/// # let action: Action<_> =
/// chars(|ch| ch.is_ascii_digit());
/// ```
#[inline]
pub fn chars<State, Heap>(
  condition: impl Fn(char) -> bool + 'static,
) -> Action<MockTokenKind<()>, State, Heap> {
  simple(move |input| {
    let mut digested = 0;
    for ch in input.rest().chars() {
      if !condition(ch) {
        break;
      }
      digested += ch.len_utf8();
    }
    digested
  })
}

/// Match chars in the range greedily.
///
/// The [`Action::head`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, chars_in_range};
/// // match all ascii digits greedily
/// # let action: Action<_> =
/// chars_in_range('0'..='9');
/// ```
#[inline]
pub fn chars_in_range<State, Heap>(
  range: impl Into<RangeInclusive<char>>,
) -> Action<MockTokenKind<()>, State, Heap> {
  let range = range.into();
  {
    let range = range.clone();
    chars(move |ch| range.contains(&ch))
  }
  .unchecked_head_in_range(range)
}

/// Match chars in the set greedily.
///
/// The [`Action::head`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, charset};
/// // match 'a' or 's' or 'd' greedily
/// # let action: Action<_> =
/// charset(['a', 's', 'd']);
/// ```
#[inline]
pub fn charset<State, Heap>(
  set: impl Into<HashSet<char>>,
) -> Action<MockTokenKind<()>, State, Heap> {
  let set = set.into();
  {
    // TODO: optimize runtime perf using lookup table
    let set = set.clone();
    chars(move |ch| set.contains(&ch))
  }
  .unchecked_head_in(set)
}

/// Match chars in the string greedily.
///
/// The [`Action::head`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, chars_in_str};
/// // match 'a' or 's' or 'd' greedily
/// # let action: Action<_> =
/// chars_in_str("asd");
/// ```
#[inline]
pub fn chars_in_str<State, Heap>(s: impl Into<String>) -> Action<MockTokenKind<()>, State, Heap> {
  charset(s.into().chars().collect::<HashSet<_>>())
}

/// Match unicode whitespaces greedily.
/// For the list of whitespaces, see https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt.
///
/// The [`Action::head`] will be set automatically.
/// # Caveats
/// The range of unicode whitespaces is from `0x0009` to `0x3000`,
/// which will cause a large lookup table when building the lexer,
/// the building time will be increased and the runtime memory usage will be increased.
///
/// You may not need to support all unicode whitespaces.
/// E.g. in JSON you only need to support `0x0009`, `0x000A`, `0x000D`, `0x0020`.
/// # Examples
/// ```
/// # use whitehole::lexer::{action::whitespaces, token::token_kind};
/// # use whitehole::lexer::LexerBuilder;
/// # #[token_kind]
/// # #[derive(Default, Clone)]
/// # enum MyKind { #[default] Anonymous }
/// # fn main() {
/// # let builder = LexerBuilder::<MyKind>::new();
/// builder.ignore_default(whitespaces());
/// # }
/// ```
#[inline]
pub fn whitespaces<State, Heap>() -> Action<MockTokenKind<()>, State, Heap> {
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

/// Match from the `open` to the `close`, including the `open` and `close`.
/// If the `close` is not found, accept all the rest.
///
/// The [`Action::head`] will be set automatically.
/// # Panics
/// Panics if the open quote is empty.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, comment};
/// // single line comment
/// # let action: Action<_> =
/// comment("//", "\n");
/// # let action: Action<_> =
/// comment("#", "\n");
/// // multi line comment
/// # let action: Action<_> =
/// comment("/*", "*/");
/// # let action: Action<_> =
/// comment("<!--", "-->");
/// ```
#[inline]
pub fn comment<State, Heap>(
  open: impl Into<String>,
  close: impl Into<String>,
) -> Action<MockTokenKind<()>, State, Heap> {
  let open: String = open.into();
  let close: String = close.into();
  let first = open.chars().next().expect("open is empty");

  simple(move |input| {
    // open mismatch
    if !input.rest().starts_with(&open) {
      return 0;
    }

    input.rest()[open.len()..]
      .find(&close)
      // if match, return total length
      .map(|i| i + open.len() + close.len())
      // if the close is not found,
      // accept all rest as the comment
      .unwrap_or(input.rest().len())
  })
  .unchecked_head_in([first])
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{ActionInput, HeadMatcher};

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

  #[test]
  fn action_utils_chars() {
    let action: Action<MockTokenKind<()>> = chars(|ch| ch.is_ascii_digit());

    // common cases
    assert_reject(&action, "abc");
    assert_accept(&action, "123abc", 3);
    assert_accept(&action, "123", 3);

    // head matcher
    assert!(action.head().is_none());
  }

  #[test]
  fn action_utils_chars_in_range() {
    let action: Action<MockTokenKind<()>> = chars_in_range('1'..='3');

    // common cases
    assert_reject(&action, "abc");
    assert_accept(&action, "123abc", 3);
    assert_accept(&action, "123", 3);

    // head matcher
    assert!(matches!(
      action.head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set == &HashSet::from(['1', '2', '3'])
    ));
  }

  #[test]
  fn action_utils_charset() {
    let action: Action<MockTokenKind<()>> = charset(['1', '3', '5']);

    // common cases
    assert_reject(&action, "abc");
    assert_accept(&action, "135abc", 3);
    assert_accept(&action, "135", 3);

    // head matcher
    assert!(matches!(
      action.head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set == &HashSet::from(['1', '3', '5'])
    ));
  }

  #[test]
  fn action_utils_chars_in_str() {
    let action: Action<MockTokenKind<()>> = chars_in_str("135");

    // common cases
    assert_reject(&action, "abc");
    assert_accept(&action, "135abc", 3);
    assert_accept(&action, "135", 3);

    // head matcher
    assert!(matches!(
      action.head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set == &HashSet::from(['1', '3', '5'])
    ));
  }

  #[test]
  fn action_utils_comment() {
    let action: Action<MockTokenKind<()>> = comment("//", "\n");

    // common cases
    let text = "// this is a comment\n";
    assert_reject(&action, "123");
    assert_reject(&action, "  // ");
    assert_reject(&action, "/");
    assert_accept(&action, text, text.len());

    // no close
    let text = "// this is a comment";
    assert_accept(&action, text, text.len());

    // head matcher
    assert!(matches!(
      action.head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'/')
    ));
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
      action.head().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == text.chars().count() && set.iter().all(|c| text.contains(*c))
    ));
  }
}
