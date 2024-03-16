use super::{SubTokenKind, TokenKindId, TokenKindIdProvider};

/// A mock struct which implements [`SubTokenKind`] and [`TokenKindIdProvider`].
/// This is useful in action utils to pass data to downstream actions.
#[derive(Debug)]
pub struct MockTokenKind<T> {
  pub data: T,
}

/// The only possible kind id of [`MockTokenKind`].
// make the only possible kind id a static const
// so that we don't need to create it every time
// and we don't need to store it in the struct
const MOCK_TOKEN_KIND_ID: TokenKindId<MockTokenKind<()>> = TokenKindId::new(0);

impl<T> MockTokenKind<T> {
  pub fn id() -> &'static TokenKindId<Self> {
    &MOCK_TOKEN_KIND_ID.cast()
  }

  pub fn new(data: T) -> Self {
    Self { data }
  }
}

impl<T> TokenKindIdProvider<MockTokenKind<T>> for MockTokenKind<T> {
  fn id(&self) -> &TokenKindId<Self> {
    &MOCK_TOKEN_KIND_ID.cast()
  }
}

impl<T> SubTokenKind<Self> for MockTokenKind<T> {
  fn kind_id() -> TokenKindId<Self> {
    MOCK_TOKEN_KIND_ID.cast().clone()
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
      MockTokenKind::<()>::kind_id(),
      MockTokenKind::<()>::id().clone()
    );
  }

  #[test]
  fn mock_token_kind_new() {
    let kind = MockTokenKind::new(42);
    assert_eq!(kind.data, 42);
  }
}
