use crate::kind::KindIdBinding;

pub type Range = std::ops::Range<usize>;

/// A range of text with a kind value.
/// # Design
/// ## There is no `Token::content` here (as a `&str`)
/// When lexing you can get the content in
/// [`AcceptedActionOutputContext::content`](crate::lexer::action::AcceptedActionOutputContext::content),
/// parse its value if needed and store the result data in [`Self::binding`].
/// The content may only be used less than once, and can be calculated manually from [`Self::range`].
/// You can calculate and cache it by yourself.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token<Kind> {
  /// The sub kind id and the kind value.
  pub binding: KindIdBinding<Kind>,
  /// The byte range of the token in the input text.
  /// This can be used to index the input text.
  /// # Example
  /// ```
  /// # use whitehole::{kind::MockKind, lexer::token::Token};
  /// let token = Token {
  ///   binding: MockKind::new(()).into(),
  ///   range: 0..5,
  /// };
  /// // index a string with the range
  /// assert_eq!(&"0123456"[token.range], "01234");
  pub range: Range,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::kind::MockKind;

  #[test]
  fn test_token() {
    let token = Token {
      binding: MockKind::new(()).into(),
      range: 0..5, // ensure we can create the range with the range syntax
    };

    // ensure the range can be used to index a string
    assert_eq!(&"0123456"[token.range], "01234");
  }
}
