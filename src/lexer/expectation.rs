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
  /// Set the expected kind of the token.
  /// Only the kind id is compared, data will be ignored.
  pub fn kind(mut self, kind: impl Into<Kind>) -> Self
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    self.kind = Some(kind.into().id());
    self
  }
}

impl<'expect_text, Kind> Expectation<'expect_text, Kind> {
  /// Set the expected text content of the token.
  pub fn text(mut self, text: impl Into<&'expect_text str>) -> Self {
    self.text = Some(text.into());
    self
  }
}
