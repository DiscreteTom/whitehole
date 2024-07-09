use crate::lexer::action::{StringLiteralError, StringList};
use std::collections::HashMap;

use super::{Escape, EscapeHandler};

/// Returns an escape handler that
/// map escape sequences to their corresponding values.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{map, Escape};
/// # enum MyError { UnnecessaryEscape }
/// # let escape_handler: EscapeHandler<MyError> =
/// map([
///  ('n', '\n'),
///  ('t', '\t'),
/// ]);
/// ```
pub fn map<CustomError>(m: impl Into<HashMap<char, char>>) -> EscapeHandler<CustomError> {
  let m = m.into();
  Box::new(move |input| match input.rest.chars().next() {
    // `input.rest` is guaranteed to be non-empty
    // so `next` is always `Some`
    None => unreachable!(),
    Some(next) => m.get(&next).map(|&value| Escape {
      digested: next.len_utf8(),
      value: value.into(),
      error: None,
    }),
  })
}

/// Returns an escape handler that
/// accept a list of strings as line continuations.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{line_continuation, Escape};
/// # enum MyError { UnnecessaryEscape }
/// // treat `"\\\r\n"` and `"\\\n"` to `''`
/// # let escape_handler: EscapeHandler<MyError> =
/// line_continuation(["\r\n", "\n"]);
pub fn line_continuation<CustomError>(ss: impl Into<StringList>) -> EscapeHandler<CustomError> {
  let ss: Vec<String> = ss.into().0;
  Box::new(move |input| {
    for s in ss.iter() {
      if input.rest.starts_with(s) {
        return Some(Escape {
          digested: s.len(),
          value: "".into(),
          error: None,
        });
      }
    }
    None
  })
}

/// Returns an escape handler that
/// accept one character as the escaped value and mark the escape as a customized error.
/// E.g. treat `'\\z'` as `'z'`.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{fallback, Escape};
/// # enum MyError { UnnecessaryEscape }
/// fallback(MyError::UnnecessaryEscape);
/// ```
pub fn fallback<CustomError: Clone + 'static>(error: CustomError) -> EscapeHandler<CustomError> {
  Box::new(move |input| match input.rest.chars().next() {
    // `input.rest` is guaranteed to be non-empty
    // so `next` is always `Some`
    None => unreachable!(),
    Some(next) => Some(Escape {
      digested: next.len_utf8(),
      value: next.into(),
      error: Some(StringLiteralError::Custom(error.clone())),
    }),
  })
}
