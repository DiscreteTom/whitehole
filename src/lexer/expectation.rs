use super::token::{SubTokenKind, TokenKindId};

#[derive(Clone, Debug, PartialEq, Eq)]
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

impl<'expect_literal, Kind> From<&'static TokenKindId<Kind>>
  for Expectation<'expect_literal, Kind>
{
  #[inline]
  fn from(id: &'static TokenKindId<Kind>) -> Self {
    Self::new().kind(id)
  }
}

impl<'expect_literal, Kind, ViaKind: SubTokenKind<TokenKind = Kind>> From<ViaKind>
  for Expectation<'expect_literal, Kind>
{
  #[inline]
  fn from(_: ViaKind) -> Self {
    Self::new().kind(ViaKind::kind_id())
  }
}

impl<'expect_literal, Kind> From<&'expect_literal str> for Expectation<'expect_literal, Kind> {
  #[inline]
  fn from(text: &'expect_literal str) -> Self {
    Self::new().literal(text)
  }
}

impl<'expect_literal, Kind> Expectation<'expect_literal, Kind> {
  /// Create a new [`Expectation`] with no expected [`kind`](Self::kind) and no expected [`literal`](Self::literal).
  #[inline]
  pub const fn new() -> Self {
    Expectation {
      kind: None,
      literal: None,
    }
  }

  /// If the [`kind`](Self::kind) is provided, the lexer will skip [`Action`](crate::lexer::action::Action)s
  /// with different [`kind`](crate::lexer::action::Action::kind) (unless [`muted`](crate::lexer::action::Action::muted)).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::token::{token_kind, SubTokenKind};
  /// # use whitehole::lexer::expectation::Expectation;
  /// #[token_kind]
  /// enum MyKind { A(String), B }
  /// # fn main() {
  /// // use kind id, useful for enum variant with associated data
  /// // so you don't need to create a new instance
  /// # let mut expectation = Expectation::new();
  /// expectation.kind(A::kind_id());
  /// // use the variant itself, useful for unit variants
  /// # let mut expectation = Expectation::new();
  /// expectation.kind(B);
  /// # }
  /// ```
  #[inline]
  pub fn kind(mut self, kind: impl Into<&'static TokenKindId<Kind>>) -> Self {
    self.kind = Some(kind.into());
    self
  }

  /// If the [`literal`](Self::literal) is provided, the lexer will skip [`Action`](crate::lexer::action::Action)s
  /// with different [`literal`](crate::lexer::action::Action::literal) (unless [`muted`](crate::lexer::action::Action::muted)).
  /// # Caveats
  /// Be ware, we are checking the [`Action::literal`](crate::lexer::action::Action::literal)
  /// *before* executing an action,
  /// not the action output's text content
  /// *after* executing an action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::expectation::Expectation;
  /// // with static string
  /// # let mut expectation = Expectation::<()>::new();
  /// expectation.literal("import");
  /// // with owned string
  /// # let mut expectation = Expectation::<()>::new();
  /// expectation.literal(&String::from("import"));
  /// ```
  #[inline]
  pub const fn literal(mut self, text: &'expect_literal str) -> Self {
    self.literal = Some(text);
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::token::SubTokenKind;
  use whitehole_macros::_token_kind;

  #[_token_kind]
  #[derive(Debug)]
  enum MyKind {
    A(()),
    B,
  }

  #[test]
  fn expectation_new_default() {
    let expectation = Expectation::<()>::default();
    assert_eq!(expectation.kind, None);
    assert_eq!(expectation.literal, None);
    let expectation = Expectation::<()>::new();
    assert_eq!(expectation.kind, None);
    assert_eq!(expectation.literal, None);
  }

  #[test]
  fn expectation_from_kind_id_or_sub_token_kind_value() {
    let expectation = Expectation::from(A::kind_id());
    assert_eq!(expectation.kind, Some(A::kind_id()));
    assert_eq!(expectation.literal, None);
    let expectation = Expectation::from(B);
    assert_eq!(expectation.kind, Some(B::kind_id()));
    assert_eq!(expectation.literal, None);
  }

  #[test]
  fn expectation_from_text() {
    let expectation = Expectation::<()>::from("text");
    assert_eq!(expectation.kind, None);
    assert_eq!(expectation.literal, Some("text"));
  }

  #[test]
  fn expectation_methods() {
    let expectation = Expectation::new();
    let expectation = expectation.kind(A::kind_id());
    assert_eq!(expectation.kind, Some(A::kind_id()));
    assert_eq!(expectation.literal, None);
    let expectation = expectation.literal("text");
    assert_eq!(expectation.kind, Some(A::kind_id()));
    assert_eq!(expectation.literal, Some("text"));
    let s = String::from("owned");
    let expectation = expectation.kind(B).literal(&s);
    assert_eq!(expectation.kind, Some(B::kind_id()));
    assert_eq!(expectation.literal, Some("owned"));
  }
}
