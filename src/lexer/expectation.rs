use crate::kind::{SubKind, SubKindId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expectation<'literal, Kind> {
  /// See [`Self::kind`].
  pub kind: Option<SubKindId<Kind>>,
  /// See [`Self::literal`].
  pub literal: Option<&'literal str>,
}

impl<'literal, Kind> Default for Expectation<'literal, Kind> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<'literal, Kind> From<SubKindId<Kind>> for Expectation<'literal, Kind> {
  #[inline]
  fn from(id: SubKindId<Kind>) -> Self {
    Self::new().kind(id)
  }
}

impl<'literal, Kind, Sub: SubKind<Kind = Kind>> From<Sub> for Expectation<'literal, Kind> {
  #[inline]
  fn from(_: Sub) -> Self {
    Self::new().kind(Sub::kind_id())
  }
}

impl<'literal, Kind> From<&'literal str> for Expectation<'literal, Kind> {
  #[inline]
  fn from(text: &'literal str) -> Self {
    Self::new().literal(text)
  }
}

impl<'literal, Kind> Expectation<'literal, Kind> {
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
  /// # use whitehole::kind::{whitehole_kind, SubKind};
  /// # use whitehole::lexer::expectation::Expectation;
  /// #[whitehole_kind]
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
  pub fn kind(mut self, kind: impl Into<SubKindId<Kind>>) -> Self {
    self.kind = Some(kind.into());
    self
  }

  /// If the [`literal`](Self::literal) is provided, the lexer will skip [`Action`](crate::lexer::action::Action)s
  /// with different [`literal`](crate::lexer::action::Action::literal) (unless [`muted`](crate::lexer::action::Action::muted)).
  /// # Caveats
  /// Be ware, we are checking the [`Action::literal`](crate::lexer::action::Action::literal)
  /// *before* executing an action,
  /// not checking the action output's text content
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
  pub const fn literal(mut self, text: &'literal str) -> Self {
    self.literal = Some(text);
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::kind::SubKind;
  use whitehole_macros::_whitehole_kind;

  #[_whitehole_kind]
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
  fn expectation_from_kind_id_or_sub_kind_value() {
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
