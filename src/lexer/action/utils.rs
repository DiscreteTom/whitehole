mod accumulator;
mod exact;
mod float;
mod integer;
mod regex;
mod string;
mod string_list;
mod word;

pub use accumulator::*;
pub use exact::*;
pub use float::*;
pub use integer::*;
pub use regex::*;
pub use string::*;
pub use string_list::*;
pub use word::*;

use super::{simple::simple, Action};
use crate::lexer::token::MockTokenKind;

/// Match unicode whitespaces greedy.
/// For the list of whitespaces, see https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt.
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::{action::whitespaces, token::{TokenKindIdBinding, token_kind}};
/// # use whitehole::lexer::LexerBuilder;
/// # #[token_kind]
/// # #[derive(Default, Clone)]
/// # enum MyKind { #[default] Anonymous }
/// # fn main() {
/// # let builder = LexerBuilder::<TokenKindIdBinding<MyKind>>::new();
/// builder.ignore_default(whitespaces());
/// # }
/// ```
pub fn whitespaces<ActionState, ErrorType>() -> Action<MockTokenKind<()>, ActionState, ErrorType> {
  simple(|input| {
    let mut i = 0;
    for ch in input.rest().chars() {
      if !ch.is_whitespace() {
        break;
      }
      i += ch.len_utf8();
    }
    i
  })
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
    '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{0020}', '\u{0085}', '\u{00A0}',
    '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}',
    '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{2028}', '\u{2029}', '\u{202F}', '\u{205F}',
    '\u{3000}',
  ])
}

/// Match from the `open` to the `close`, including the `open` and `close`.
/// If the `close` is not found, accept all the rest.
///
/// [`Action::head_matcher`] will be set automatically.
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
pub fn comment<ActionState, ErrorType>(
  open: impl Into<String>,
  close: impl Into<String>,
) -> Action<MockTokenKind<()>, ActionState, ErrorType> {
  let open: String = open.into();
  let close: String = close.into();
  let first = open.chars().next().unwrap();

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

// TODO: add string & numeric utils?

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
  fn action_utils_comment() {
    let action: Action<MockTokenKind<()>> = comment("//", "\n");

    // common cases
    let text = "// this is a comment\n";
    assert_reject(&action, "123");
    assert_reject(&action, "  // ");
    assert_reject(&action, "/");
    assert_accept(&action, &text, text.len());

    // no close
    let text = "// this is a comment";
    assert_accept(&action, &text, text.len());

    // head matcher
    assert!(matches!(
      action.head_matcher().as_ref().unwrap(),
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
      action.head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == text.chars().count() && set.iter().all(|c| text.contains(*c))
    ));
  }
}
