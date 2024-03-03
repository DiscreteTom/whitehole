use std::{hash::Hash, marker::PhantomData};

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
  pub fn new(index: usize) -> Self {
    TokenKindId(index, PhantomData)
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
impl<TokenKindType> PartialOrd for TokenKindId<TokenKindType> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.0.partial_cmp(&other.0)
  }
}
impl<TokenKindType> Ord for TokenKindId<TokenKindType> {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.0.cmp(&other.0)
  }
}
