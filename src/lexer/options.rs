use super::core::lex::expectation::Expectation;

pub struct LexerLexOptions<'expect_text, Kind> {
  pub peek: bool,
  pub expectation: Expectation<'expect_text, Kind>,
}

impl<'expect_text, Kind> From<Expectation<'expect_text, Kind>>
  for LexerLexOptions<'expect_text, Kind>
{
  fn from(expectation: Expectation<'expect_text, Kind>) -> Self {
    LexerLexOptions {
      peek: false,
      expectation,
    }
  }
}

impl<'expect_text, Kind> Default for LexerLexOptions<'expect_text, Kind> {
  fn default() -> Self {
    LexerLexOptions {
      peek: false,
      expectation: Expectation::default(),
    }
  }
}

impl<'expect_text, Kind> LexerLexOptions<'expect_text, Kind> {
  pub fn peek(mut self, peek: impl Into<bool>) -> Self {
    self.peek = peek.into();
    self
  }

  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_text, Kind>>) -> Self {
    let Expectation { text, kind } = expectation.into();
    if let Some(text) = text {
      self.expectation = self.expectation.text(text);
    }
    if let Some(kind) = kind {
      self.expectation = self.expectation.kind(kind);
    }
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole_macros::TokenKind;

  #[derive(TokenKind)]
  enum MyKind {
    UnitField,
    UnnamedField(i32),
    // NamedField { _a: i32 },
  }

  #[test]
  fn default() {
    let default = LexerLexOptions::<()>::default();
    assert_eq!(default.peek, false);
    assert_eq!(default.expectation.kind, None);
    assert_eq!(default.expectation.text, None);
  }

  #[test]
  fn peek() {
    let e = LexerLexOptions::<()>::default().peek(true);
    assert_eq!(e.peek, true);
    // overwrite
    assert_eq!(e.peek(false).peek, false);
  }

  #[test]
  fn expect_kind() {
    let e = LexerLexOptions::default().expect(MyKind::UnitField);
    assert!(matches!(e.expectation.kind, Some(MyKind::UnitField)),);
    // overwrite
    assert!(matches!(
      e.expect(MyKind::UnnamedField(0)).expectation.kind,
      Some(MyKind::UnnamedField(..))
    ),);
  }

  #[test]
  fn expect_text() {
    let e = LexerLexOptions::<()>::default().expect("abc");
    assert_eq!(e.expectation.text, Some("abc"));
    // overwrite
    assert_eq!(e.expect("def").expectation.text, Some("def"));
  }

  #[test]
  fn expect_kind_and_text() {
    let e = LexerLexOptions::default()
      .expect("abc")
      .expect(MyKind::UnitField);
    assert!(matches!(e.expectation.kind, Some(MyKind::UnitField)),);
    assert_eq!(e.expectation.text, Some("abc"));
  }
}
