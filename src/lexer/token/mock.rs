use super::{TokenKind, TokenKindId};
use std::collections::HashSet;

/// A mock struct which implements [`TokenKind`]
/// and only has one possible kind id.
/// This is useful in action utils to pass data to downstream actions.
pub struct MockTokenKind<T> {
  id: TokenKindId<MockTokenKind<T>>,
  pub data: T,
}

impl<T> MockTokenKind<T> {
  /// Return the only possible kind id.
  pub fn id() -> TokenKindId<MockTokenKind<T>> {
    // TODO: make the id static to prevent creation?
    TokenKindId::new(0)
  }

  /// Return a [`HashSet`] containing the only possible kind id.
  pub fn possible_kinds() -> HashSet<TokenKindId<MockTokenKind<T>>> {
    HashSet::from([Self::id()])
  }

  pub fn new(data: T) -> Self {
    Self {
      id: Self::id(),
      data,
    }
  }
}

impl<T> TokenKind<MockTokenKind<T>> for MockTokenKind<T> {
  fn id(&self) -> &TokenKindId<MockTokenKind<T>> {
    &self.id
  }
}
