/// Errors when lexing a string.
#[derive(Clone, Debug, PartialEq)]
pub enum StringLiteralError<CustomError> {
  /// The string is unterminated.
  /// E.g. `"hello`
  Unterminated,
  /// An escape sequence is not handled by any escape handler.
  UnhandledEscape,
  Custom(CustomError),
}
