mod hex;
mod simple;

pub use hex::*;
pub use simple::*;

use super::{
  PartialStringBody, PartialStringBodyValue, StringBodyMatcherInput, StringBodyOptions,
  StringLiteralError,
};

pub struct Escape<Value, CustomError> {
  /// The number of bytes that have been digested as the escape body
  /// (not include the escape starter).
  pub digested: usize,
  /// The value of the partial string body.
  pub value: Value,
  pub error: Option<StringLiteralError<CustomError>>,
}

pub type EscapeHandler<Value, CustomError> =
  Box<dyn Fn(&StringBodyMatcherInput) -> Option<Escape<Value, CustomError>>>;

impl<Value: PartialStringBodyValue + 'static, CustomError: 'static, BodyAcc>
  StringBodyOptions<Value, CustomError, BodyAcc>
{
  /// Append a string body matcher to match escape sequences.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{StringBodyOptions, fallback, line_continuation};
  /// # enum MyError { UnnecessaryEscape }
  /// # let options =
  /// StringBodyOptions::new().escape('\\', [
  ///   line_continuation(["\r\n", "\n"]),
  ///   fallback(MyError::UnnecessaryEscape)
  /// ]);
  /// ```
  pub fn escape(
    mut self,
    starter: char,
    handlers: impl Into<Vec<EscapeHandler<Value, CustomError>>>,
  ) -> Self {
    let handlers = handlers.into();
    self.matchers.push(Box::new(move |input| {
      if input.next != starter {
        // not an escape starter
        return None;
      }

      match StringBodyMatcherInput::new(&input.rest[starter.len_utf8()..]) {
        None => {
          // unterminated string
          // treat the escape starter as a normal char
          Some(PartialStringBody {
            digested: starter.len_utf8(),
            value: Value::from_char(starter),
            close: true,
            error: Some(StringLiteralError::Unterminated),
          })
        }
        Some(input) => {
          for handler in handlers.iter() {
            if let Some(escape) = handler(&input) {
              return Some(PartialStringBody {
                digested: starter.len_utf8() + escape.digested,
                value: escape.value,
                close: false,
                error: escape.error,
              });
            }
          }
          // else, no escape handler accepted,
          // treat the escape starter as a normal char
          Some(PartialStringBody {
            digested: starter.len_utf8(),
            value: Value::from_char(starter),
            close: false,
            error: Some(StringLiteralError::UnhandledEscape),
          })
        }
      }
    }));
    self
  }
}
