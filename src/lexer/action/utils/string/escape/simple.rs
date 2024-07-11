use super::{Escape, EscapeHandler};
use crate::lexer::action::{StringList, StringLiteralError};
use std::collections::HashMap;

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
  Box::new(move |input| {
    m.get(&input.next).map(|&value| Escape {
      digested: input.next.len_utf8(),
      value: value.into(),
      error: None,
    })
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
  Box::new(move |input| {
    Some(Escape {
      digested: input.next.len_utf8(),
      value: input.next.into(),
      error: Some(StringLiteralError::Custom(error.clone())),
    })
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::StringBodyMatcherInput;

  fn escape_checker_factory(h: EscapeHandler<()>, err: bool) -> impl Fn(&str, Option<&str>) {
    move |src, value| match h(&StringBodyMatcherInput::new(src).unwrap()) {
      Some(escape) => {
        assert_eq!(escape.value, value.unwrap());
        if err {
          assert!(matches!(escape.error, Some(StringLiteralError::Custom(()))));
        } else {
          assert!(escape.error.is_none());
        }
      }
      None => {
        assert!(value.is_none());
      }
    }
  }

  #[test]
  fn test_map() {
    let check = escape_checker_factory(
      map([
        ('n', '\n'),
        ('r', '\r'),
        ('t', '\t'),
        ('0', '\0'),
        ('\\', '\\'),
      ]),
      false,
    );
    check(r"n", "\n".into());
    check(r"r", "\r".into());
    check(r"t", "\t".into());
    check(r"0", "\0".into());
    check(r"\", "\\".into());
    check(r"a", None);
  }

  #[test]
  fn test_line_continuation() {
    let check = escape_checker_factory(line_continuation(["\r\n", "\n"]), false);
    check("\r\n", "".into());
    check("\n", "".into());
    check(r"a", None);
  }

  #[test]
  fn test_fallback() {
    let check = escape_checker_factory(fallback(()), true);
    check(r"a", "a".into());
  }
}
