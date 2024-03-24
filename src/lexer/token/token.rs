pub type Range = std::ops::Range<usize>;

// make all fields public so the user can destruct the struct and get the fields
pub struct Token<'text, Kind, ErrorType> {
  /// The kind and the binding data.
  pub kind: Kind,
  pub content: &'text str,
  /// The range of the token in the input string.
  /// The range can be used to index the input string.
  /// # Example
  /// ```
  /// # use whitehole::lexer::token::Token;
  /// let token = Token {
  ///   kind: (),
  ///   content: "hello",
  ///   range: 0..5,
  ///   error: None::<()>,
  /// };
  /// // indexing some string with the range
  /// assert_eq!(&"0123456"[token.range], "01234");
  pub range: Range,
  pub error: Option<ErrorType>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_token() {
    let token = Token {
      kind: (),
      content: "hello",
      range: 0..5, // ensure we can create the range with the range syntax
      error: None::<()>,
    };

    // ensure the range is able to index a string
    assert_eq!(&"0123456"[token.range], "01234");
  }
}
