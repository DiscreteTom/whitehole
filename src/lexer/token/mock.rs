use super::{SubTokenKind, TokenKindId, TokenKindIdProvider};

/// This implements [`SubTokenKind`] and [`TokenKindIdProvider`],
/// and only have one possible token kind id value(and the value is 0).
/// This is useful as a placeholder or data carrier.
/// # Examples
/// ```
/// use whitehole::lexer::token::{
///   MockTokenKind, TokenKindIdProvider, SubTokenKind, TokenKindId
/// };
/// assert_eq!(MockTokenKind::<()>::kind_id(), TokenKindId::new(0));
/// assert_eq!(MockTokenKind::new(42).id(), &TokenKindId::new(0));
/// ```
#[derive(Debug)] // `T` should impl `Debug`
pub struct MockTokenKind<T> {
  pub data: T,
}

/// The only possible kind id of [`MockTokenKind`].
// make the only possible kind id a static const
// so that we don't need to create it every time
// and we don't need to store it in the struct
const MOCK_TOKEN_KIND_ID: TokenKindId<MockTokenKind<()>> = TokenKindId::new(0);

// we store `TokenKindType` only for type checking and the `PhantomData` is zero sized
// so we can safely cast self to another type
fn cast_mock_token_kind_id<'a, T>() -> &'a TokenKindId<T> {
  unsafe { std::mem::transmute(&MOCK_TOKEN_KIND_ID) }
}

impl<T> MockTokenKind<T> {
  pub fn new(data: T) -> Self {
    Self { data }
  }
}

impl<T> TokenKindIdProvider<Self> for MockTokenKind<T> {
  fn id(&self) -> &TokenKindId<Self> {
    cast_mock_token_kind_id()
  }
}

impl<T> SubTokenKind<Self> for MockTokenKind<T> {
  // TODO: make the kind id static?
  fn kind_id() -> TokenKindId<Self> {
    TokenKindId::new(0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mock_token_kind_new() {
    let kind = MockTokenKind::new(42);
    assert_eq!(kind.data, 42);
  }

  #[test]
  fn mock_token_kind_id() {
    let kind = MockTokenKind::new(());
    assert_eq!(kind.id(), &TokenKindId::new(0));
    assert_eq!(MockTokenKind::<()>::kind_id(), TokenKindId::new(0));
  }

  #[test]
  fn token_kind_id_cast() {
    fn cast_to_unit<T>(id: &TokenKindId<T>) -> &TokenKindId<()> {
      unsafe { std::mem::transmute(id) }
    }
    let id0 = TokenKindId::new(0) as TokenKindId<()>;
    let id1 = TokenKindId::new(0) as TokenKindId<i32>;
    let id2 = TokenKindId::new(0) as TokenKindId<Box<i32>>;
    let id3 = TokenKindId::new(0) as TokenKindId<Option<i32>>;
    let id4 = TokenKindId::new(0) as TokenKindId<Result<i32, i32>>;

    let ids = [
      cast_to_unit(&id0),
      cast_to_unit(&id1),
      cast_to_unit(&id2),
      cast_to_unit(&id3),
      cast_to_unit(&id4),
    ];

    // ensure their memory layout is the same
    for i in 0..ids.len() {
      for j in 0..ids.len() {
        assert_eq!(ids[i], ids[j]);
      }
    }
  }
}
