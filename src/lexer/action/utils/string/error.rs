/// Errors when lexing a string.
pub enum Error<CustomError> {
  Unterminated,
  UnhandledEscape,
  Custom(CustomError),
}
