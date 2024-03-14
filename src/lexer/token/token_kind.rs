use std::{collections::HashSet, hash::Hash, marker::PhantomData};

pub trait TokenKind<TokenKindType> {
  fn id(&self) -> TokenKindId<TokenKindType>;
}

/// The unique id of a token kind.
/// Usually we use enum variants as token kinds, and the id is the variant's index.
#[derive(Debug)]
pub struct TokenKindId<TokenKindType>(pub usize, PhantomData<TokenKindType>);

impl<TokenKindType: TokenKind<TokenKindType>> From<TokenKindType> for TokenKindId<TokenKindType> {
  fn from(kind: TokenKindType) -> Self {
    kind.id()
  }
}

impl<TokenKindType> TokenKindId<TokenKindType> {
  pub fn new(id: usize) -> Self {
    TokenKindId(id, PhantomData)
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

/// A mock struct which implements [`TokenKind`].
/// This is useful in action utils to pass data to downstream actions.
pub struct MockTokenKind<T> {
  pub data: T,
}

impl<T> MockTokenKind<T> {
  /// Return the only possible kind id for [`MockTokenKind`].
  pub fn id() -> TokenKindId<MockTokenKind<T>> {
    TokenKindId::new(0)
  }

  /// Return a [`HashSet`] containing the only possible kind id for [`MockTokenKind`].
  pub fn possible_kinds() -> HashSet<TokenKindId<MockTokenKind<T>>> {
    HashSet::from([Self::id()])
  }
}

impl<T> TokenKind<MockTokenKind<T>> for MockTokenKind<T> {
  fn id(&self) -> TokenKindId<MockTokenKind<T>> {
    TokenKindId::new(0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole_macros::_TokenKind;
  use MyKind::*;

  #[derive(_TokenKind)]
  enum MyKind {
    UnitField,
    UnnamedField(i32),
    NamedField { _a: i32 },
  }

  #[test]
  fn token_kind_id() {
    assert_eq!(UnitField.id().0, 0);
    assert_eq!(UnnamedField(42).id().0, 1);
    assert_eq!(NamedField { _a: 1 }.id().0, 2);
  }
}
