use super::token::{TokenKindId, TokenKindIdProvider};

pub struct Expectation<'expect_literal, Kind: 'static> {
  /// See [`Self::kind`].
  pub kind: Option<&'static TokenKindId<Kind>>,
  /// See [`Self::literal`].
  pub literal: Option<&'expect_literal str>,
}

impl<'expect_literal, Kind> Default for Expectation<'expect_literal, Kind> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<'expect_literal, Kind> From<&'static TokenKindId<Kind>> for Expectation<'expect_literal, Kind>
where
  Kind: TokenKindIdProvider<Kind>,
{
  fn from(id: &'static TokenKindId<Kind>) -> Self {
    Expectation {
      kind: Some(id),
      literal: None,
    }
  }
}

impl<'expect_literal, Kind> From<&'expect_literal str> for Expectation<'expect_literal, Kind> {
  fn from(text: &'expect_literal str) -> Self {
    Expectation {
      kind: None,
      literal: Some(text),
    }
  }
}

impl<'expect_literal, Kind> Expectation<'expect_literal, Kind> {
  #[inline]
  pub const fn new() -> Self {
    Expectation {
      kind: None,
      literal: None,
    }
  }

  /// If the [`kind`](Self::kind) is provided, the lexer will skip [`Action`](crate::lexer::action::Action)s
  /// with different [`kind_id`](crate::lexer::action::Action::kind) (unless [`muted`](crate::lexer::action::Action::muted)).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::token::{token_kind, SubTokenKind};
  /// # use whitehole::lexer::expectation::Expectation;
  /// #[token_kind]
  /// enum MyKind { A }
  /// // use kind id
  /// # fn main() {
  /// # let mut expectation = Expectation::default();
  /// expectation.kind(A::kind_id());
  /// // for unit enum variant, you can use the variant itself
  /// # let mut expectation = Expectation::default();
  /// expectation.kind(A);
  /// # }
  /// ```
  pub fn kind(mut self, kind: impl Into<&'static TokenKindId<Kind>>) -> Self
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    self.kind = Some(kind.into());
    self
  }
}

impl<'expect_literal, Kind> Expectation<'expect_literal, Kind> {
  /// If the [`literal`](Self::literal) is provided, the lexer will skip [`Action`](crate::lexer::action::Action)s
  /// with different [`literal`](crate::lexer::action::Action::literal) (unless [`muted`](crate::lexer::action::Action::muted)).
  /// # Caveats
  /// Be ware, we are checking the [`Action::literal`](crate::lexer::action::Action::literal)
  /// before executing an action,
  /// not the token's text content
  /// after executing an action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::token::{token_kind};
  /// # use whitehole::lexer::expectation::Expectation;
  /// # #[token_kind]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut expectation = Expectation::<MyKind>::default();
  /// expectation.literal("import");
  /// # }
  /// ```
  pub fn literal(mut self, text: impl Into<&'expect_literal str>) -> Self {
    self.literal = Some(text.into());
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
    assert_eq!(expectation.literal, None);
  }

  #[test]
  fn expectation_from_kind_id() {
    let expectation = Expectation::from(A::kind_id());
    assert_eq!(expectation.kind, Some(A::kind_id()));
    assert_eq!(expectation.literal, None);
  }

  #[test]
  fn expectation_from_text() {
    let expectation = Expectation::<MyKind>::from("text");
    assert_eq!(expectation.kind, None);
    assert_eq!(expectation.literal, Some("text"));
  }
}
