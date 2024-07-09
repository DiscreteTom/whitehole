use super::{Escape, EscapeHandler};
use crate::lexer::action::StringLiteralError;

pub enum HexEscapeError {
  TooShort,
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
  Box::new(move |input| match input.rest.chars().next() {
    // `input.rest` is guaranteed to be non-empty
    // so `next` is always `Some`
    None => unreachable!(),
    Some(next) => {
      // check prefix
      if next != options.prefix {
        return None;
      }

      let mut value = 0;
      let mut i = 0;
      let mut digested = options.prefix.len_utf8();
      for c in input.rest.chars().skip(1) {
        match c.to_digit(16) {
          // not enough digits
          None => {
            return Some(Escape {
              digested: options.prefix.len_utf8(), // only digest the prefix
              value: '\0'.into(),                  // treat as '\0'
              error: Some(StringLiteralError::Custom(HexEscapeError::TooShort)),
            });
          }
          Some(digit) => {
            value = value * 16 + digit;
            i += 1;
            digested += c.len_utf8();
            if i == options.length {
              match char::from_u32(value) {
                // invalid unicode
                None => {
                  return Some(Escape {
                    digested: options.prefix.len_utf8(), // only digest the prefix
                    value: '\0'.into(),                  // treat as '\0'
                    error: Some(StringLiteralError::Custom(HexEscapeError::InvalidUnicode)),
                  });
                }
                Some(res) => {
                  return Some(Escape {
                    digested,
                    value: res.into(),
                    error: None,
                  });
                }
              }
            }
          }
        }
      }

      // reach to the end of the string
      Some(Escape {
        digested: options.prefix.len_utf8(), // only digest the prefix
        value: '\0'.into(),                  // treat as '\0'
        error: Some(StringLiteralError::Custom(HexEscapeError::TooShort)),
      })
    }
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
