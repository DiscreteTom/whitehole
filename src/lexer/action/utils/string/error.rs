/// Errors when lexing a string.
#[derive(Clone, Debug)]
pub enum StringLiteralError<CustomError> {
  Unterminated,
  UnhandledEscape,
  Custom(CustomError),
}
