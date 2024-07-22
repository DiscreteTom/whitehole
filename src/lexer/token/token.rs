pub type Range = std::ops::Range<usize>;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct Token<Kind, ErrorType> {
  /// The kind and the binding data.
  pub kind: Kind,
  /// The byte range of the token in the input text.
  /// This can be used to index the input text.
  /// # Example
  /// ```
  /// # use whitehole::lexer::token::Token;
  /// let token = Token {
  ///   kind: (),
  ///   range: 0..5,
  ///   error: None::<()>,
  /// };
  /// // index a string with the range
  /// assert_eq!(&"0123456"[token.range], "01234");
  pub range: Range,
  /// If `Some`, the token is an error token.
  /// Error tokens will be collected during the lexing process.
  pub error: Option<ErrorType>,
  // we don't store `token.content` here (as a `&str`).
  // `token.content` may only be used less than once, and can be calculated from `token.range`.
  // users can calculate and cache it by themselves, we don't do unnecessary work.
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_token() {
    let token = Token {
      kind: (),
      range: 0..5, // ensure we can create the range with the range syntax
      error: None::<()>,
    };

    // ensure the range can be used to index a string
    assert_eq!(&"0123456"[token.range], "01234");
  }
}
