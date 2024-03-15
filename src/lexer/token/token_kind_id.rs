use std::{hash::Hash, marker::PhantomData};

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

// TODO: tests
