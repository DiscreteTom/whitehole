mod hex;
mod simple;

pub use hex::*;
pub use simple::*;

use super::{
  PartialStringBody, PartialStringBodyValue, StringBodyMatcherInput, StringBodyOptions,
  StringLiteralError,
};

pub struct Escape<CustomError> {
  /// The number of bytes that have been digested as the escape body
  /// (not include the escape starter).
  pub digested: usize,
  /// The value of the partial string body.
  pub value: String,
  pub error: Option<StringLiteralError<CustomError>>,
}

pub type EscapeHandler<CustomError> =
  Box<dyn Fn(&StringBodyMatcherInput) -> Option<Escape<CustomError>>>;

impl<Value: PartialStringBodyValue, CustomError: 'static> StringBodyOptions<Value, CustomError> {
  /// Append a string body matcher to match escape sequences.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{StringBodyOptions, fallback};
  /// # enum MyError { UnnecessaryEscape }
  /// # let options =
  /// StringBodyOptions::new().escape('\\', vec![
  ///   fallback(MyError::UnnecessaryEscape)
  /// ]);
  /// ```
  pub fn escape(mut self, starter: char, handlers: Vec<EscapeHandler<CustomError>>) -> Self {
    self.matchers.push(Box::new(move |input| {
      if input.next != starter {
        // not an escape starter
        return None;
      }

      match StringBodyMatcherInput::new(input.text, input.start + starter.len_utf8()) {
        None => {
          // unterminated string
          // treat the escape starter as a normal char
          Some(PartialStringBody {
            digested: starter.len_utf8(),
            value: Value::from_char(&starter),
            close: true,
            error: Some(StringLiteralError::Unterminated),
          })
        }
        Some(input) => {
          for handler in handlers.iter() {
            if let Some(escape) = handler(&input) {
              return Some(PartialStringBody {
                digested: starter.len_utf8() + escape.digested,
                value: Value::from_str(&escape.value),
                close: false,
                error: escape.error,
              });
            }
          }
          // else, no escape handler accepted,
          // treat the escape starter as a normal char
          Some(PartialStringBody {
            digested: starter.len_utf8(),
            value: Value::from_char(&starter),
            close: false,
            error: Some(StringLiteralError::UnhandledEscape),
          })
        }
      }
    }));
    self
  }
}
