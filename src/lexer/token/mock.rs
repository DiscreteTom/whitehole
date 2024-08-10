use super::{SubTokenKind, TokenKindId, TokenKindIdBinding};

/// This implements [`SubTokenKind`] and `Into<TokenKindIdBinding<MockTokenKind<T>>>`
/// and only has one possible token kind id value.
/// This is useful as a placeholder or data carrier.
/// # Examples
/// ```
/// use whitehole::lexer::token::{MockTokenKind, SubTokenKind, TokenKindIdBinding};
///
/// let v1: TokenKindIdBinding<MockTokenKind<i32>> = MockTokenKind::new(42).into();
/// let v2: TokenKindIdBinding<MockTokenKind<bool>> = MockTokenKind::new(true).into();
///
/// assert_eq!(v1.id(), MockTokenKind::kind_id());
/// assert_eq!(v2.id(), MockTokenKind::kind_id());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct MockTokenKind<T> {
  /// The data carried by the [`MockTokenKind`].
  pub data: T,
}

impl<T> MockTokenKind<T> {
  /// Create a new instance with the given data.
  #[inline]
  pub const fn new(data: T) -> Self {
    Self { data }
  }
}

impl<T> SubTokenKind for MockTokenKind<T> {
  type TokenKind = Self;
  #[inline]
  fn kind_id() -> TokenKindId<Self::TokenKind> {
    // the only possible token kind id value
    TokenKindId::new(0)
  }
}

impl<T> Into<TokenKindIdBinding<MockTokenKind<T>>> for MockTokenKind<T> {
  #[inline]
  fn into(self) -> TokenKindIdBinding<MockTokenKind<T>> {
    TokenKindIdBinding::new(self)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mock_token_kind_new() {
    assert_eq!(MockTokenKind::new(42).data, 42);
    assert_eq!(MockTokenKind::new(()).data, ());
  }

  #[test]
  fn mock_token_kind_id() {
    assert_eq!(MockTokenKind::<u32>::kind_id().value(), 0);
    assert_eq!(MockTokenKind::<Box<u32>>::kind_id().value(), 0);
  }

  #[test]
  fn mock_token_kind_into_binding() {
    let v1: TokenKindIdBinding<MockTokenKind<i32>> = MockTokenKind::new(42).into();
    let v2: TokenKindIdBinding<MockTokenKind<bool>> = MockTokenKind::new(true).into();

    assert_eq!(v1.id(), MockTokenKind::kind_id());
    assert_eq!(v2.id(), MockTokenKind::kind_id());
  }
}
