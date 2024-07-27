use super::{SubTokenKind, TokenKindId, TokenKindIdProvider};
use std::mem::transmute;

/// This implements [`SubTokenKind`] and [`TokenKindIdProvider`],
/// and only has one possible token kind id value.
/// This is useful as a placeholder or data carrier.
/// # Examples
/// ```
/// use whitehole::lexer::token::{MockTokenKind, SubTokenKind, TokenKindIdProvider};
/// assert_eq!(MockTokenKind::new(42).id(), MockTokenKind::kind_id());
/// assert_eq!(MockTokenKind::new(()).id(), MockTokenKind::kind_id());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct MockTokenKind<T> {
  pub data: T,
}

/// The only possible kind id of [`MockTokenKind`].
const MOCK_TOKEN_KIND_ID: TokenKindId<MockTokenKind<()>> = TokenKindId::new(0, "");

/// Since all `TokenKindId<MockTokenKind<T>>` have the same memory layout,
/// it should be safe to cast it to any `TokenKindId<MockTokenKind<T>>`.
#[inline]
const fn cast_mock_token_kind_id<T>() -> &'static TokenKindId<MockTokenKind<T>> {
  unsafe { transmute(&MOCK_TOKEN_KIND_ID) }
}

impl<T> MockTokenKind<T> {
  /// Create a new `MockTokenKind` with the given data.
  #[inline]
  pub const fn new(data: T) -> Self {
    Self { data }
  }
}

impl<T> TokenKindIdProvider for MockTokenKind<T> {
  type TokenKind = Self;
  #[inline]
  fn id(&self) -> &'static TokenKindId<Self::TokenKind> {
    cast_mock_token_kind_id()
  }
}

impl<T> SubTokenKind for MockTokenKind<T> {
  type TokenKind = Self;
  #[inline]
  fn kind_id() -> &'static TokenKindId<Self::TokenKind> {
    cast_mock_token_kind_id()
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
    assert_eq!(MockTokenKind::new(42).id(), MockTokenKind::kind_id());
    assert_eq!(MockTokenKind::new(()).id(), MockTokenKind::kind_id());
  }

  #[test]
  fn token_kind_id_cast() {
    fn cast_to_unit<T>(id: &TokenKindId<T>) -> &TokenKindId<()> {
      unsafe { std::mem::transmute(id) }
    }

    let id0 = TokenKindId::new(0, "") as TokenKindId<()>;
    let id1 = TokenKindId::new(0, "") as TokenKindId<i32>;
    let id2 = TokenKindId::new(0, "") as TokenKindId<Box<i32>>;
    let id3 = TokenKindId::new(0, "") as TokenKindId<Option<i32>>;
    let id4 = TokenKindId::new(0, "") as TokenKindId<Result<i32, i32>>;

    let ids = [
      cast_to_unit(&id0),
      cast_to_unit(&id1),
      cast_to_unit(&id2),
      cast_to_unit(&id3),
      cast_to_unit(&id4),
      cast_to_unit(MockTokenKind::new(()).id()),
      cast_to_unit(MockTokenKind::new(123).id()),
      cast_to_unit(MockTokenKind::new(Box::new(123)).id()),
      cast_to_unit(MockTokenKind::new(Some(123)).id()),
    ];

    // ensure their memory layout is the same
    for i in 0..ids.len() {
      for j in 0..ids.len() {
        assert_eq!(ids[i], ids[j]);
      }
    }
  }
}
