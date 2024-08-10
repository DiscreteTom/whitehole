use super::TokenKindIdBinding;

pub type Range = std::ops::Range<usize>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token<Kind> {
  /// The token kind value and the id of the token kind.
  pub binding: TokenKindIdBinding<Kind>,
  /// The byte range of the token in the input text.
  /// This can be used to index the input text.
  /// # Example
  /// ```
  /// # use whitehole::lexer::token::{Token, MockTokenKind};
  /// let token = Token {
  ///   binding: MockTokenKind::new(()).into(),
  ///   range: 0..5,
  /// };
  /// // index a string with the range
  /// assert_eq!(&"0123456"[token.range], "01234");
  pub range: Range,
  // we don't store `token.content` here (as a `&str`).
  // when lexing users can get the content in the lexing context,
  // parse its value if needed and store the result data in `self.binding.kind`.
  // `token.content` may only be used less than once, and can be calculated from `token.range`.
  // users can calculate and cache it by themselves, we don't do unnecessary work.
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::token::MockTokenKind;

  #[test]
  fn test_token() {
    let token = Token {
      binding: MockTokenKind::new(()).into(),
      range: 0..5, // ensure we can create the range with the range syntax
    };

    // ensure the range can be used to index a string
    assert_eq!(&"0123456"[token.range], "01234");
  }
}
