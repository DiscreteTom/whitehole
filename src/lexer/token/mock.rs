use super::{TokenKind, TokenKindId};
use std::collections::HashSet;

/// A mock struct which implements [`TokenKind`]
/// and only has one possible kind id.
/// This is useful in action utils to pass data to downstream actions.
pub struct MockTokenKind<T> {
  id: TokenKindId<MockTokenKind<T>>,
  // the struct only have one possible id,
  // so even the data is mutable the binding is not broken
  // so we make the data public
  pub data: T,
}

impl<T> MockTokenKind<T> {
  /// Return the only possible kind id.
  pub fn id() -> TokenKindId<MockTokenKind<T>> {
    // TODO: make the id static to prevent creation?
    TokenKindId::new(0)
  }

  pub fn new(data: T) -> Self {
    Self {
      id: Self::id(),
      data,
    }
  }
}

impl<T> TokenKind<MockTokenKind<T>> for MockTokenKind<T> {
  type TargetType = Self;

  /// Return the only possible kind id.
  fn id(&self) -> &TokenKindId<Self> {
    &self.id
  }

  /// Return a [`HashSet`] containing the only possible kind id.
  fn possible_kinds() -> HashSet<TokenKindId<Self::TargetType>> {
    HashSet::from([Self::id()])
  }
}
