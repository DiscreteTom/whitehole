use std::{collections::HashSet, hash::Hash, marker::PhantomData, ops::Deref};

pub trait TokenKind<TokenKindType> {
  fn id(&self) -> &TokenKindId<TokenKindType>;
}

/// The unique id of a token kind value.
/// Usually we use enum variants as token kind values, and the id is the variant's index.
#[derive(Debug)]
pub struct TokenKindId<TokenKindType>(pub usize, PhantomData<TokenKindType>);

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

/// Bind the token kind value with an [`TokenKindId`].
/// This is readonly to make sure the binding is not broken.
pub struct TokenKindIdBinding<TokenKindType> {
  id: TokenKindId<TokenKindType>,
  value: TokenKindType,
}

// value is private and need to be accessed by `value()`
// so for convenience we impl Deref
impl<TokenKindType> Deref for TokenKindIdBinding<TokenKindType> {
  type Target = TokenKindType;
  fn deref(&self) -> &Self::Target {
    &self.value
  }
}
// don't impl DerefMut because we want this to be readonly

impl<TokenKindType> TokenKindIdBinding<TokenKindType> {
  pub fn new(id: usize, value: TokenKindType) -> Self {
    Self {
      value,
      id: TokenKindId::new(id),
    }
  }

  pub fn id(&self) -> &TokenKindId<TokenKindType> {
    &self.id
  }
  pub fn value(&self) -> &TokenKindType {
    &self.value
  }

  /// Consume self and take the value out.
  pub fn take(self) -> TokenKindType {
    self.value
  }
}

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
