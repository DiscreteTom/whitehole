use super::token::{TokenKindId, TokenKindIdProvider};

pub struct Expectation<'expect_text, Kind: 'static> {
  pub kind: Option<&'static TokenKindId<Kind>>,
  pub text: Option<&'expect_text str>,
}

impl<'expect_text, Kind> Default for Expectation<'expect_text, Kind> {
  fn default() -> Self {
    Expectation {
      kind: None,
      text: None,
    }
  }
}

impl<'expect_text, Kind> From<&'static TokenKindId<Kind>> for Expectation<'expect_text, Kind>
where
  Kind: TokenKindIdProvider<Kind>,
{
  fn from(id: &'static TokenKindId<Kind>) -> Self {
    Expectation {
      kind: Some(id),
      text: None,
    }
  }
}

impl<'expect_text, Kind> From<&'expect_text str> for Expectation<'expect_text, Kind> {
  fn from(text: &'expect_text str) -> Self {
    Expectation {
      kind: None,
      text: Some(text),
    }
  }
}

impl<'expect_text, Kind> Expectation<'expect_text, Kind> {
  /// Set the expected kind id of the token.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::token::{token_kind};
  /// # use whitehole::lexer::expectation::Expectation;
  /// #[token_kind]
  /// enum MyKind { A }
  /// // use kind id
  /// # let mut expectation = Expectation::default();
  /// expectation.kind(A::kind_id());
  /// // for unit enum variant, you can use the variant itself
  /// expectation.kind(A);
  /// ```
  pub fn kind(mut self, kind: impl Into<&'static TokenKindId<Kind>>) -> Self
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    self.kind = Some(kind.into());
    self
  }
}

impl<'expect_text, Kind> Expectation<'expect_text, Kind> {
  /// Set the expected text content of the token.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::token::{token_kind};
  /// # use whitehole::lexer::expectation::Expectation;
  /// # #[token_kind]
  /// # enum MyKind { A }
  /// # let mut expectation = Expectation::<MyKind>::default();
  /// expectation.text("text");
  /// ```
  pub fn text(mut self, text: impl Into<&'expect_text str>) -> Self {
    self.text = Some(text.into());
    self
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::token::SubTokenKind;

  use super::*;
  use whitehole_macros::_token_kind;

  #[_token_kind]
  #[derive(Debug)]
  enum MyKind {
    A,
  }

  #[test]
  fn expectation_default() {
    let expectation = Expectation::<MyKind>::default();
    assert_eq!(expectation.kind, None);
    assert_eq!(expectation.text, None);
  }

  #[test]
  fn expectation_from_kind_id() {
    let expectation = Expectation::from(A::kind_id());
    assert_eq!(expectation.kind, Some(A::kind_id()));
    assert_eq!(expectation.text, None);
  }

  #[test]
  fn expectation_from_text() {
    let expectation = Expectation::<MyKind>::from("text");
    assert_eq!(expectation.kind, None);
    assert_eq!(expectation.text, Some("text"));
  }
}
