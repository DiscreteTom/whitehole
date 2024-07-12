/// Errors when lexing a string.
#[derive(Clone, Debug, PartialEq)]
pub enum StringLiteralError<CustomError> {
  Unterminated,
  UnhandledEscape,
  Custom(CustomError),
}
