use super::{Escape, EscapeHandler};
use crate::lexer::action::StringLiteralError;

pub enum HexEscapeError {
  /// The hex sequence is shorter than [`HexEscapeOptions::length`].
  /// E.g. `"\x1"`.
  TooShort,
  /// The hex sequence does not represent a valid unicode character.
  InvalidUnicode,
}

pub struct HexEscapeOptions {
  /// The prefix of the hex escape.
  /// Defaults to `'x'`.
  pub prefix: char,
  /// The number of hex digit chars to match.
  /// Defaults to `2`.
  pub length: usize,
}

impl Default for HexEscapeOptions {
  fn default() -> Self {
    Self {
      prefix: 'x',
      length: 2,
    }
  }
}

impl HexEscapeOptions {
  /// Create a new `HexEscapeOptions` with the default settings for unicode escapes.
  /// Set the prefix to `'u'` and the length to `4`.
  pub fn unicode() -> Self {
    Self {
      prefix: 'u',
      length: 4,
    }
  }

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
}

// TODO: comments
pub fn hex() -> EscapeHandler<HexEscapeError> {
  hex_with_options(HexEscapeOptions::default())
}

// TODO: comments
pub fn hex_with(
  options_builder: impl FnOnce(HexEscapeOptions) -> HexEscapeOptions,
) -> EscapeHandler<HexEscapeError> {
  hex_with_options(options_builder(HexEscapeOptions::default()))
}

// TODO: comments
pub fn hex_with_options(options: HexEscapeOptions) -> EscapeHandler<HexEscapeError> {
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
          return escape_error(options.prefix.len_utf8(), '\0', HexEscapeError::TooShort);
        }
        Some(digit) => {
          value = value * 16 + digit;
          i += 1;
          digested += c.len_utf8();
          if i == options.length {
            return match char::from_u32(value) {
              // invalid unicode
              None => escape_error(
                options.prefix.len_utf8(),
                '\0',
                HexEscapeError::InvalidUnicode,
              ),
              Some(res) => Some(Escape {
                digested,
                value: res.into(),
                error: None,
              }),
            };
          }
        }
      }
    }

    // reach to the end of the string
    escape_error(options.prefix.len_utf8(), '\0', HexEscapeError::TooShort)
  })
}

// TODO: comments
pub fn unicode() -> EscapeHandler<HexEscapeError> {
  hex_with_options(HexEscapeOptions::unicode())
}

// TODO: comments
pub fn unicode_with(
  options_builder: impl FnOnce(HexEscapeOptions) -> HexEscapeOptions,
) -> EscapeHandler<HexEscapeError> {
  hex_with_options(options_builder(HexEscapeOptions::unicode()))
}

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

pub struct CodePointEscapeOptions {
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
  pub max_length: usize,
}

pub fn code_point_with_options(
  options: CodePointEscapeOptions,
) -> EscapeHandler<CodePointEscapeError> {
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
          return escape_error(options.prefix.len_utf8(), '\0', CodePointEscapeError::Empty);
        }
        digested += c.len_utf8();
        return match char::from_u32(value) {
          // invalid unicode
          None => escape_error(
            options.prefix.len_utf8(),
            '\0',
            CodePointEscapeError::InvalidUnicode,
          ),
          Some(res) => Some(Escape {
            digested,
            value: res.into(),
            error: None,
          }),
        };
      }

      if i == options.max_length {
        // reach the maximum length but not closed
        return escape_error(
          options.prefix.len_utf8(),
          '\0',
          CodePointEscapeError::Overlong,
        );
      }

      match c.to_digit(16) {
        // bad hex digit
        None => {
          return escape_error(
            options.prefix.len_utf8(),
            '\0',
            CodePointEscapeError::InvalidChar,
          )
        }
        Some(digit) => {
          value = value * 16 + digit;
          i += 1;
          digested += c.len_utf8();
        }
      }
    }

    // reach to the end of the string
    escape_error(
      options.prefix.len_utf8(),
      '\0',
      CodePointEscapeError::Unterminated,
    )
  })
}

fn escape_error<CustomError>(
  digested: usize,
  value: impl Into<String>,
  e: CustomError,
) -> Option<Escape<CustomError>> {
  Some(Escape {
    digested,
    value: value.into(),
    error: Some(StringLiteralError::Custom(e)),
  })
}
