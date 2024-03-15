use std::{hash::Hash, marker::PhantomData};

/// The unique id of a token kind value.
/// Usually we use enum variants as token kind values, and the id is the variant's index.
#[derive(Debug)]
pub struct TokenKindId<TokenKindType>(pub usize, PhantomData<TokenKindType>);

impl<TokenKindType> TokenKindId<TokenKindType> {
  pub fn new(id: usize) -> Self {
    TokenKindId(id, PhantomData)
  }

  /// Cast the token kind id to another token kind id.
  /// This is only used internally in [`MockTokenKind`](super::mock::MockTokenKind).
  pub(crate) fn cast<T>(&self) -> &TokenKindId<T> {
    // we store TokenKindType only for type checking and the PhantomData is zero sized
    // so we can safely cast self to another type
    unsafe { &*(self as *const TokenKindId<TokenKindType> as *const TokenKindId<T>) }
  }
}

impl<TokenKindType> PartialEq for TokenKindId<TokenKindType> {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}
impl<TokenKindType> Eq for TokenKindId<TokenKindType> {}

impl<TokenKindType> Hash for TokenKindId<TokenKindType> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.0.hash(state);
  }
}

impl<TokenKindType> Clone for TokenKindId<TokenKindType> {
  fn clone(&self) -> Self {
    TokenKindId(self.0, PhantomData)
  }
}

impl<TokenKindType> Copy for TokenKindId<TokenKindType> {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn token_kind_id_new() {
    let id = TokenKindId::new(42) as TokenKindId<()>;
    assert_eq!(id.0, 42);
  }

  #[test]
  fn token_kind_id_eq() {
    let id1 = TokenKindId::new(42) as TokenKindId<()>;
    let id2 = TokenKindId::new(42) as TokenKindId<()>;
    assert_eq!(id1, id2);
  }

  #[test]
  fn token_kind_id_clone() {
    let id = TokenKindId::new(42) as TokenKindId<()>;
    let id_clone = id.clone();
    assert_eq!(id, id_clone);
  }

  #[test]
  fn token_kind_id_cast() {
    fn cast_to_unit<T>(id: &TokenKindId<T>) -> &TokenKindId<()> {
      id.cast()
    }
    let ids = [
      cast_to_unit(&TokenKindId::new(0) as &TokenKindId<()>),
      cast_to_unit(&TokenKindId::new(0) as &TokenKindId<i32>),
      cast_to_unit(&TokenKindId::new(0) as &TokenKindId<Box<i32>>),
      cast_to_unit(&TokenKindId::new(0) as &TokenKindId<Option<i32>>),
      cast_to_unit(&TokenKindId::new(0) as &TokenKindId<Result<i32, i32>>),
    ];
    // ensure their memory layout is the same
    for i in 0..ids.len() {
      for j in 0..ids.len() {
        assert_eq!(ids[i], ids[j]);
      }
    }
  }
}
