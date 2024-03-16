use super::{TokenKind, TokenKindId, TokenKindIdProvider};
use std::collections::HashSet;

/// A mock struct which implements [`TokenKind`] and [`TokenKindIdProvider`],
/// and only has one possible kind id.
/// This is useful in action utils to pass data to downstream actions.
#[derive(Debug)]
pub struct MockTokenKind<T> {
  // the struct only have one possible id,
  // so even the data is mutable, the binding is not broken
  // so we make the data public
  pub data: T,
}

/// The only possible kind id of [`MockTokenKind`].
// make the only possible kind id a static const
// so that we don't need to create it every time
// and we don't need to store it in the struct
const MOCK_TOKEN_KIND_ID: TokenKindId<MockTokenKind<()>> = TokenKindId::new(0);

impl<T> MockTokenKind<T> {
  /// Return the only possible kind id.
  pub fn id() -> &'static TokenKindId<Self> {
    &MOCK_TOKEN_KIND_ID.cast()
  }

  pub fn new(data: T) -> Self {
    Self { data }
  }
}

impl<T> TokenKindIdProvider<MockTokenKind<T>> for MockTokenKind<T> {
  /// Return the only possible kind id.
  fn id(&self) -> &TokenKindId<Self> {
    &MOCK_TOKEN_KIND_ID.cast()
  }
}

impl<T> TokenKind<Self> for MockTokenKind<T> {
  /// Return a [`HashSet`] containing the only possible kind id.
  fn possible_kinds() -> HashSet<TokenKindId<Self>> {
    HashSet::from([MOCK_TOKEN_KIND_ID.cast().clone()])
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mock_token_kind_id() {
    let kind = MockTokenKind { data: () };
    assert_eq!(kind.id().0, 0);
    assert_eq!(MockTokenKind::<()>::id().0, 0);
  }

  #[test]
  fn mock_token_kind_possible_kinds() {
    assert_eq!(
      MockTokenKind::<()>::possible_kinds(),
      HashSet::from([MockTokenKind::<()>::id().clone()])
    );
  }

  #[test]
  fn mock_token_kind_new() {
    let kind = MockTokenKind::new(42);
    assert_eq!(kind.data, 42);
  }
}
