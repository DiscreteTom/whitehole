use super::{Escape, EscapeHandler};
use crate::lexer::action::StringLiteralError;

pub struct HexEscapeOptions<CustomError> {
  /// The prefix of the hex escape.
  /// Defaults to `'x'`.
  pub prefix: char,
  /// The number of hex digit chars to match.
  /// Defaults to `2`.
  pub length: usize,
  /// If [`Some`], invalid hex escapes will be marked with this error
  /// and be accepted. If [`None`], invalid hex escapes will be rejected.
  pub error: Option<CustomError>,
}

impl<CustomError> Default for HexEscapeOptions<CustomError> {
  fn default() -> Self {
    Self {
      prefix: 'x',
      length: 2,
      error: None,
    }
  }
}

impl<CustomError> HexEscapeOptions<CustomError> {
  /// Create a new `HexEscapeOptions` with the default settings for unicode escapes.
  /// Set the prefix to `'u'` and the length to `4`.
  pub fn unicode() -> Self {
    Self {
      prefix: 'u',
      length: 4,
      error: None,
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

  /// Set the error to be used for invalid hex escapes.
  pub fn error(mut self, error: CustomError) -> Self {
    self.error = Some(error);
    self
  }
}

// TODO: comments
pub fn hex<CustomError: Clone + 'static>() -> EscapeHandler<CustomError> {
  hex_with_options(HexEscapeOptions::default())
}

// TODO: comments
pub fn hex_with<CustomError: Clone + 'static>(
  options_builder: impl FnOnce(HexEscapeOptions<CustomError>) -> HexEscapeOptions<CustomError>,
) -> EscapeHandler<CustomError> {
  hex_with_options(options_builder(HexEscapeOptions::default()))
}

// TODO: comments
pub fn hex_with_options<CustomError: Clone + 'static>(
  options: HexEscapeOptions<CustomError>,
) -> EscapeHandler<CustomError> {
  let error = options.error.map(|e| StringLiteralError::Custom(e));
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
          None => break,
          Some(digit) => {
            value = value * 16 + digit;
            i += 1;
            digested += c.len_utf8();
            if i == options.length {
              match char::from_u32(value) {
                // invalid unicode
                None => break,
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

      // reach to the end of the string or encounter a non-hex char
      // or the value is not a valid unicode,
      // set error if provided
      // otherwise, return None to reject the escape
      error.clone().map(|error| Escape {
        digested: options.prefix.len_utf8(),
        value: '\0'.into(),
        error: Some(error),
      })
    }
  })
}

// TODO: comments
pub fn unicode<CustomError: Clone + 'static>() -> EscapeHandler<CustomError> {
  hex_with_options(HexEscapeOptions::unicode())
}

// TODO: comments
pub fn unicode_with<CustomError: Clone + 'static>(
  options_builder: impl FnOnce(HexEscapeOptions<CustomError>) -> HexEscapeOptions<CustomError>,
) -> EscapeHandler<CustomError> {
  hex_with_options(options_builder(HexEscapeOptions::unicode()))
}
