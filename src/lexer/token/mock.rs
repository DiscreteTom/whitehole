use super::SubTokenKind;

/// This implements [`SubTokenKind`]
/// and the [`TokenKindId::value`](crate::lexer::token::TokenKindId::value)
/// will always be `0`.
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
  const VARIANT_INDEX: usize = 0;
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::token::TokenKindIdBinding;

  #[test]
  fn mock_token_kind_new() {
    assert_eq!(MockTokenKind::new(42).data, 42);
    assert_eq!(MockTokenKind::new("123").data, "123");
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
