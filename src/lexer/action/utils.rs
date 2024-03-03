use super::{simple::simple, Action};

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

/// Match a string exactly, ***NO LOOKAHEAD***.
/// The head matcher will be set automatically.
pub fn exact<ActionState, ErrorType>(s: impl Into<String>) -> Action<(), ActionState, ErrorType> {
  let s: String = s.into();
  let first = s.chars().next().unwrap();
  simple(move |input| {
    if input.rest().starts_with(&s) {
      s.len()
    } else {
      0
    }
  })
  .head_in([first])
}

/// Match a word, lookahead one char to ensure there is a word boundary or end of input.
/// The head matcher will be set automatically.
pub fn word<ActionState: 'static, ErrorType: 'static>(
  s: impl Into<String>,
) -> Action<(), ActionState, ErrorType> {
  exact(s).reject_if(|ctx| {
    ctx
      .output
      .rest()
      .chars()
      .next()
      // if next char exists and is alphanumeric or `_` then reject
      .map(|c| c.is_alphanumeric() || c == '_')
      // if next char does not exist, then accept
      .unwrap_or(false)
  })
}
