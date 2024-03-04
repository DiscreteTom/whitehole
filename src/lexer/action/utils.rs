mod string_list;

pub use string_list::*;

use super::{simple::simple, Action};
use std::collections::HashSet;

/// Match unicode whitespaces greedy.
/// The head matcher will be set automatically.
///
/// For the list of whitespaces, see https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt.
pub fn whitespaces<ActionState, ErrorType>() -> Action<(), ActionState, ErrorType> {
  // TODO: benchmark this vs regex `^\s+`
  simple(|input| {
    let mut digested = 0;
    // TODO: maybe someday we can get a `&char` instead of a `char` here
    for (i, c) in input.rest().char_indices() {
      if c.is_whitespace() {
        digested = i + c.len_utf8();
      } else {
        break;
      }
    }
    digested
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
  .head_in([
    '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{0020}', '\u{0085}', '\u{00A0}',
    '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}',
    '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{2028}', '\u{2029}', '\u{202F}', '\u{205F}',
    '\u{3000}',
  ])
}

/// Match from the `open` to the `close`, including the `open` and `close`.
/// If the `close` is not found, accept all rest as the comment.
/// The head matcher will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, comment};
/// // single line comment
/// # let action: Action<()> =
/// comment("//", "\n");
/// # let action: Action<()> =
/// comment("#", "\n");
/// // multi line comment
/// # let action: Action<()> =
/// comment("/*", "*/");
/// # let action: Action<()> =
/// comment("<!--", "-->");
/// ```
pub fn comment<ActionState, ErrorType>(
  open: impl Into<String>,
  close: impl Into<String>,
) -> Action<(), ActionState, ErrorType> {
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
  .head_in([first])
}

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
pub fn exact<ActionState, ErrorType>(
  ss: impl Into<StringList>,
) -> Action<(), ActionState, ErrorType> {
  let ss: Vec<String> = ss.into().0;

  if ss.len() == 0 {
    panic!("empty string list");
  }

  // optimize for single string
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

/// Match one of the provided words,
/// ***LOOKAHEAD*** one char to ensure there is a word boundary
/// (alphanumeric or `_`) or end of input after the word.
/// Stop at the first match.
/// The head matcher will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, word};
/// # let action: Action<()> =
/// // single word
/// word("a");
/// # let action: Action<()> =
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
/// # let action: Action<()> =
/// word(["ab", "a"]);
/// ```
pub fn word<ActionState: 'static, ErrorType: 'static>(
  ss: impl Into<StringList>,
) -> Action<(), ActionState, ErrorType> {
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
    .head_in([head]);
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
  .head_in(heads)
}

// TODO: add tests
// TODO: add string & numeric utils

#[cfg(test)]
mod tests {
  use crate::lexer::action::{ActionInput, ActionInputRestHeadMatcher};

  use super::*;
  #[test]
  fn action_utils_whitespaces() {
    let action: Action<()> = whitespaces();

    // common cases
    let text = " \n\t";
    assert_eq!(
      action
        .exec(&mut ActionInput::new(text, 0, &mut ()))
        .unwrap()
        .digested,
      text.len()
    );

    // full cases
    let text: String = [
      '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{0020}', '\u{0085}',
      '\u{00A0}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}',
      '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{2028}',
      '\u{2029}', '\u{202F}', '\u{205F}', '\u{3000}',
    ]
    .into_iter()
    .collect();
    assert_eq!(
      action
        .exec(&mut ActionInput::new(text.as_str(), 0, &mut ()))
        .unwrap()
        .digested,
      text.len()
    );

    // head matcher
    assert!(matches!(
      action.head_matcher().as_ref().unwrap(),
      ActionInputRestHeadMatcher::OneOf(set) if set.len() == text.chars().count() && set.iter().all(|c| text.contains(*c))
    ));
  }

  #[test]
  fn action_utils_comment() {
    let action: Action<()> = comment("//", "\n");

    // common cases
    let text = "// this is a comment\n";
    assert_eq!(
      action
        .exec(&mut ActionInput::new(text, 0, &mut ()))
        .unwrap()
        .digested,
      text.len()
    );

    // no close
    let text = "// this is a comment";
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
      ActionInputRestHeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'/')
    ));
  }
}
