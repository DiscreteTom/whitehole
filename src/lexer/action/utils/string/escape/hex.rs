use super::{Escape, EscapeHandler};
use crate::lexer::action::{PartialStringBodyValue, StringLiteralError};

#[derive(Clone, PartialEq, Debug)]
pub enum HexEscapeError {
  /// The hex sequence is shorter than [`HexEscapeOptions::length`].
  /// E.g. `"\x1"`.
  TooShort,
  /// The hex sequence does not represent a valid unicode character.
  InvalidUnicode,
}

pub struct HexEscapeOptions<CustomError> {
  /// The prefix of the hex escape.
  /// Defaults to `'x'`.
  pub prefix: char,
  /// The number of hex digit chars to match.
  /// Defaults to `2`.
  pub length: usize,
  /// Map [`HexEscapeError`] to a custom error.
  pub error_mapper: Box<dyn Fn(HexEscapeError) -> CustomError>,
}

impl Default for HexEscapeOptions<HexEscapeError> {
  #[inline]
  fn default() -> Self {
    Self {
      prefix: 'x',
      length: 2,
      error_mapper: Box::new(|e| e),
    }
  }
}

impl HexEscapeOptions<HexEscapeError> {
  /// Create a new `HexEscapeOptions` with the default settings for unicode escapes.
  /// Set the prefix to `'u'` and the length to `4`.
  pub fn unicode() -> Self {
    Self {
      prefix: 'u',
      length: 4,
      error_mapper: Box::new(|e| e),
    }
  }
}

impl<CustomError> HexEscapeOptions<CustomError> {
  /// Set the prefix of the hex escape.
  pub fn prefix(mut self, prefix: char) -> Self {
    self.prefix = prefix;
    self
  }

  /// Set the number of hex digit chars to match.
  pub fn length(mut self, length: usize) -> Self {
    self.length = length;
    self
  }

  /// Set [`Self::error_mapper`].
  pub fn error<NewError: 'static>(
    self,
    error_mapper: impl Fn(HexEscapeError) -> NewError + 'static,
  ) -> HexEscapeOptions<NewError> {
    HexEscapeOptions {
      prefix: self.prefix,
      length: self.length,
      error_mapper: Box::new(error_mapper),
    }
  }
}

// TODO: comments
pub fn hex<Value: PartialStringBodyValue>() -> EscapeHandler<Value, HexEscapeError> {
  hex_with_options(HexEscapeOptions::default())
}

// TODO: comments
pub fn hex_with<Value: PartialStringBodyValue, CustomError: 'static>(
  options_builder: impl FnOnce(HexEscapeOptions<HexEscapeError>) -> HexEscapeOptions<CustomError>,
) -> EscapeHandler<Value, CustomError> {
  hex_with_options(options_builder(HexEscapeOptions::default()))
}

// TODO: comments
pub fn hex_with_options<Value: PartialStringBodyValue, CustomError: 'static>(
  options: HexEscapeOptions<CustomError>,
) -> EscapeHandler<Value, CustomError> {
  Box::new(move |input| {
    // check prefix
    if !input.rest.starts_with(options.prefix) {
      return None;
    }

    let mut value = 0;
    let mut i = 0;
    let mut digested = options.prefix.len_utf8();
    for c in input.rest.chars().skip(1) {
      match c.to_digit(16) {
        // not enough digits
        None => {
          return escape_error(HexEscapeError::TooShort, &options.error_mapper);
        }
        Some(digit) => {
          value = value * 16 + digit;
          i += 1;
          digested += c.len_utf8();
          if i == options.length {
            return match char::from_u32(value) {
              // invalid unicode
              None => escape_error(HexEscapeError::InvalidUnicode, &options.error_mapper),
              Some(res) => Some(Escape {
                digested,
                value: Value::from_char(res),
                error: None,
              }),
            };
          }
        }
      }
    }

    // reach to the end of the string
    escape_error(HexEscapeError::TooShort, &options.error_mapper)
  })
}

// TODO: comments
/// # Caveat
/// This should be placed after [`code_point`] to match the `u` prefix
/// after [`code_point`] matches the `u{` prefix.
pub fn unicode<Value: PartialStringBodyValue>() -> EscapeHandler<Value, HexEscapeError> {
  hex_with_options(HexEscapeOptions::unicode())
}

// TODO: comments
/// # Caveat
/// This should be placed after [`code_point`] to match the `u` prefix
/// after [`code_point`] matches the `u{` prefix.
pub fn unicode_with<Value: PartialStringBodyValue, CustomError: 'static>(
  options_builder: impl FnOnce(HexEscapeOptions<HexEscapeError>) -> HexEscapeOptions<CustomError>,
) -> EscapeHandler<Value, CustomError> {
  hex_with_options(options_builder(HexEscapeOptions::unicode()))
}

#[derive(Clone, PartialEq, Debug)]
pub enum CodePointEscapeError {
  /// E.g. `"\u{}"`
  Empty,
  /// The hex sequence does not represent a valid unicode character.
  InvalidUnicode,
  /// The hex sequence contains non-hex digit characters.
  /// E.g. `"\u{z}"`
  InvalidChar,
  /// The hex sequence is longer than [`CodePointEscapeOptions::max_length`].
  Overlong,
  /// The hex sequence is not terminated.
  /// E.g. `"\u{1`
  Unterminated,
}

pub struct CodePointEscapeOptions<CustomError> {
  /// The prefix of the code point escape.
  /// Defaults to `'u'`.
  pub prefix: char,
  /// The open char of the code point escape body.
  /// Defaults to `'{'`.
  pub open: char,
  /// The close char of the code point escape body.
  /// Defaults to `'}'`.
  pub close: char,
  /// The maximum number of hex digit chars to match.
  /// Defaults to `6`.
  pub max_length: usize,
  /// Map [`CodePointEscapeError`] to a custom error.
  pub error_mapper: Box<dyn Fn(CodePointEscapeError) -> CustomError>,
}

impl Default for CodePointEscapeOptions<CodePointEscapeError> {
  #[inline]
  fn default() -> Self {
    Self {
      prefix: 'u',
      open: '{',
      close: '}',
      max_length: 6,
      error_mapper: Box::new(|e| e),
    }
  }
}

impl<CustomError> CodePointEscapeOptions<CustomError> {
  /// Set the prefix of the code point escape.
  pub fn prefix(mut self, prefix: char) -> Self {
    self.prefix = prefix;
    self
  }

  /// Set the open char of the code point escape body.
  pub fn open(mut self, open: char) -> Self {
    self.open = open;
    self
  }

  /// Set the close char of the code point escape body.
  pub fn close(mut self, close: char) -> Self {
    self.close = close;
    self
  }

  /// Set the maximum number of hex digit chars to match.
  pub fn max(mut self, max_length: usize) -> Self {
    self.max_length = max_length;
    self
  }

  /// Set [`Self::error_mapper`].
  pub fn error<NewError: 'static>(
    self,
    error_mapper: impl Fn(CodePointEscapeError) -> NewError + 'static,
  ) -> CodePointEscapeOptions<NewError> {
    CodePointEscapeOptions {
      prefix: self.prefix,
      open: self.open,
      close: self.close,
      max_length: self.max_length,
      error_mapper: Box::new(error_mapper),
    }
  }
}

// TODO: comments
/// # Caveat
/// This should be placed before [`unicode`] to match the `u{` prefix
/// before [`unicode`] matches the `u` prefix.
pub fn code_point<Value: PartialStringBodyValue>() -> EscapeHandler<Value, CodePointEscapeError> {
  code_point_with_options(CodePointEscapeOptions::default())
}

// TODO: comments
/// # Caveat
/// This should be placed before [`unicode`] to match the `u{` prefix
/// before [`unicode`] matches the `u` prefix.
pub fn code_point_with<Value: PartialStringBodyValue, CustomError: 'static>(
  options_builder: impl FnOnce(
    CodePointEscapeOptions<CodePointEscapeError>,
  ) -> CodePointEscapeOptions<CustomError>,
) -> EscapeHandler<Value, CustomError> {
  code_point_with_options(options_builder(CodePointEscapeOptions::default()))
}

// TODO: comments
/// # Caveat
/// This should be placed before [`unicode`] to match the `u{` prefix.
pub fn code_point_with_options<Value: PartialStringBodyValue, CustomError: 'static>(
  options: CodePointEscapeOptions<CustomError>,
) -> EscapeHandler<Value, CustomError> {
  let mut prefix = options.prefix.to_string();
  prefix.push(options.open);

  Box::new(move |input| {
    // check prefix
    if !input.rest.starts_with(&prefix) {
      return None;
    }

    let mut value = 0;
    let mut i = 0;
    let mut digested = prefix.len();
    for c in input.rest.chars().skip(2) {
      if c == options.close {
        if i == 0 {
          // no body
          return escape_error(CodePointEscapeError::Empty, &options.error_mapper);
        }
        digested += c.len_utf8();
        return match char::from_u32(value) {
          // invalid unicode
          None => escape_error(CodePointEscapeError::InvalidUnicode, &options.error_mapper),
          Some(res) => Some(Escape {
            digested,
            value: Value::from_char(res),
            error: None,
          }),
        };
      }

      if i == options.max_length {
        // reach the maximum length but not closed
        return escape_error(CodePointEscapeError::Overlong, &options.error_mapper);
      }

      match c.to_digit(16) {
        // bad hex digit
        None => return escape_error(CodePointEscapeError::InvalidChar, &options.error_mapper),
        Some(digit) => {
          value = value * 16 + digit;
          i += 1;
          digested += c.len_utf8();
        }
      }
    }

    // reach to the end of the string
    escape_error(CodePointEscapeError::Unterminated, &options.error_mapper)
  })
}

fn escape_error<Value: PartialStringBodyValue, FromError, ToError>(
  e: FromError,
  error_mapper: &dyn Fn(FromError) -> ToError,
) -> Option<Escape<Value, ToError>> {
  Some(Escape {
    digested: 0,             // don't digest body, only the prefix will be digested
    value: Value::default(), // empty value
    error: Some(StringLiteralError::Custom(error_mapper(e))),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::StringBodyMatcherInput;
  use std::fmt::Debug;

  fn escape_checker_factory<E: PartialEq + Debug>(
    h: EscapeHandler<String, E>,
  ) -> impl Fn(&str, Option<&str>, Option<E>) {
    move |src, value, err| match h(&StringBodyMatcherInput::new(src).unwrap()) {
      Some(escape) => {
        assert_eq!(escape.value, value.unwrap());
        match err {
          Some(e) => {
            assert_eq!(escape.error, Some(StringLiteralError::Custom(e)));
          }
          None => {
            assert!(escape.error.is_none());
          }
        }
      }
      None => {
        assert!(value.is_none());
      }
    }
  }

  #[test]
  fn test_hex_with_options() {
    let check = escape_checker_factory(hex_with_options(HexEscapeOptions::default()));
    // wrong prefix
    check("a", None, None);
    // not enough digits
    check("xz", "".into(), HexEscapeError::TooShort.into());
    // reach to the end of the string
    check("x1", "".into(), HexEscapeError::TooShort.into());
    // normal
    check("x1f", "\x1f".into(), None);

    #[derive(PartialEq, Debug)]
    struct MyError(HexEscapeError);

    let check = escape_checker_factory(hex_with_options(
      HexEscapeOptions::default()
        .prefix(':')
        .length(6)
        .error(MyError),
    ));
    // wrong prefix
    check("a", None, None);
    // not enough digits
    check(":z", "".into(), MyError(HexEscapeError::TooShort).into());
    // invalid unicode
    check(
      ":110000",
      "".into(),
      MyError(HexEscapeError::InvalidUnicode).into(),
    );
    // reach to the end of the string
    check(":1", "".into(), MyError(HexEscapeError::TooShort).into());
    // normal
    check(":01111f", "\u{01111f}".into(), None);
  }

  #[test]
  fn test_unicode() {
    let check = escape_checker_factory(unicode_with(|o| o));
    // wrong prefix
    check("a", None, None);
    // not enough digits
    check("uz", "".into(), HexEscapeError::TooShort.into());
    // reach to the end of the string
    check("u1", "".into(), HexEscapeError::TooShort.into());
    // normal
    check("u1fff", "\u{1fff}".into(), None);
  }

  #[test]
  fn test_code_point_with_options() {
    let check = escape_checker_factory(code_point_with_options(CodePointEscapeOptions::default()));
    // wrong prefix
    check("uu", None, None);
    // no body
    check("u{}", "".into(), CodePointEscapeError::Empty.into());
    // invalid unicode
    check(
      "u{110000}",
      "".into(),
      CodePointEscapeError::InvalidUnicode.into(),
    );
    // overlong
    check(
      "u{11111111111111111111",
      "".into(),
      CodePointEscapeError::Overlong.into(),
    );
    // bad hex digit
    check("u{z}", "".into(), CodePointEscapeError::InvalidChar.into());
    // not terminated
    check("u{1", "".into(), CodePointEscapeError::Unterminated.into());
    // normal
    check("u{1f}", "\u{1f}".into(), None);

    #[derive(PartialEq, Debug)]
    struct MyError(CodePointEscapeError);

    let check = escape_checker_factory(code_point_with_options(
      CodePointEscapeOptions::default()
        .prefix(':')
        .open('[')
        .close(']')
        .max(8)
        .error(MyError),
    ));
    // wrong prefix
    check("uu", None, None);
    // no body
    check(
      ":[]",
      "".into(),
      MyError(CodePointEscapeError::Empty).into(),
    );
    // invalid unicode
    check(
      ":[110000]",
      "".into(),
      MyError(CodePointEscapeError::InvalidUnicode).into(),
    );
    // overlong
    check(
      ":[11111111111111111111",
      "".into(),
      MyError(CodePointEscapeError::Overlong).into(),
    );
    // bad hex digit
    check(
      ":[z]",
      "".into(),
      MyError(CodePointEscapeError::InvalidChar).into(),
    );
    // not terminated
    check(
      ":[1",
      "".into(),
      MyError(CodePointEscapeError::Unterminated).into(),
    );
    // normal
    check(":[1f]", "\u{1f}".into(), None);
  }
}
