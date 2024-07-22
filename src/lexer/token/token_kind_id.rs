use std::{
  fmt::Debug,
  hash::{Hash, Hasher},
  marker::PhantomData,
};

/// The unique id of a sub token kind.
/// Usually we use enum variants as sub token kinds, and the id is the variant's index.
/// The id and the value can be bound together by [`TokenKindIdBinding`](super::TokenKindIdBinding).
/// The bindings can be auto generated by applying [`token_kind`](crate::lexer::token::token_kind)
/// to the token kind enum.
/// # Examples
/// ```
/// use whitehole::lexer::token::{token_kind, TokenKindId, TokenKindIdBinding, TokenKindIdProvider};
///
/// #[token_kind]
/// #[derive(Debug)]
/// enum MyKind { A, B }
/// // struct `A` and `B` are generated by the macro
/// // and implement `Into<TokenKindIdBinding<MyKind>>`
///
/// # fn main() {
/// let a: TokenKindIdBinding<MyKind> = A.into();
/// let b: TokenKindIdBinding<MyKind> = B.into();
/// assert_eq!(a.id(), &TokenKindId::new(0));
/// assert_eq!(b.id(), &TokenKindId::new(1));
/// # }
/// ```
/// # Design
/// ## Why not just use [`std::mem::Discriminant`]?
/// `Discriminant` is good, it can be used to get the unique id of an enum variant value,
/// so with `Discriminant` we don't need [`TokenKindIdBinding`](super::TokenKindIdBinding) anymore.
/// However `Discriminant` requires an instance of the enum to construct it,
/// so when using [`Action::select`](crate::lexer::action::Action::select),
/// or in the implementations of [`SubTokenKind`](super::SubTokenKind),
/// or in expectational lexing,
/// we need to construct the variant value first, just to get the id,
/// which is neither necessary nor convenient.
/// ## Why not just use [`std::any::TypeId`]?
/// - We hope the id is type-sensitive (that's why there is a [`PhantomData`] in the struct),
/// `TypeId` is not.
/// - Currently `TypeId` use 128 bits to represent the type, which is too large for our purpose.
#[derive(Debug)]
pub struct TokenKindId<TokenKindType>(usize, PhantomData<TokenKindType>);

impl<TokenKindType> TokenKindId<TokenKindType> {
  pub const fn new(id: usize) -> Self {
    TokenKindId(id, PhantomData)
  }
}

// manually implement these traits to avoid `TokenKindType`
// being `Clone`, `Copy`, `Eq`, `PartialEq`, `Hash
impl<TokenKindType> Clone for TokenKindId<TokenKindType> {
  fn clone(&self) -> Self {
    TokenKindId(self.0, PhantomData)
  }
}
impl<TokenKindType> Copy for TokenKindId<TokenKindType> {}
impl<TokenKindType> PartialEq for TokenKindId<TokenKindType> {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}
impl<TokenKindType> Eq for TokenKindId<TokenKindType> {}
impl<TokenKindType> Hash for TokenKindId<TokenKindType> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.hash(state);
  }
}

/// Calling [`TokenKindIdProvider::id`] to get the [`TokenKindId`] of a token kind value.
/// Usually we use [`TokenKindIdBinding`](super::TokenKindIdBinding) to bind the id and the value together,
/// which already implement this trait.
/// The bindings can be auto generated by applying [`token_kind`](super::token_kind)
/// to the token kind enum.
/// # Examples
/// ```
/// use whitehole::lexer::token::{token_kind, TokenKindId, TokenKindIdBinding, TokenKindIdProvider};
///
/// #[token_kind]
/// #[derive(Debug)]
/// enum MyKind { A, B }
/// // struct `A` and `B` are generated by the macro
/// // and implement `Into<TokenKindIdBinding<MyKind>>`
///
/// # fn main() {
/// let a: TokenKindIdBinding<MyKind> = A.into();
/// let b: TokenKindIdBinding<MyKind> = B.into();
/// assert_eq!(a.id(), &TokenKindId::new(0));
/// assert_eq!(b.id(), &TokenKindId::new(1));
/// # }
/// ```
pub trait TokenKindIdProvider<TokenKindType> {
  /// The token kind id of this token kind value.
  /// See [`TokenKindId`].
  fn id(&self) -> &'static TokenKindId<TokenKindType>; // use a static reference to avoid creating a new one every time
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashSet;
  use whitehole_macros::_token_kind;

  #[_token_kind]
  #[derive(Debug)]
  enum MyKind {
    A,
  }

  #[test]
  fn token_kind_id_new() {
    let id = TokenKindId::new(42) as TokenKindId<MyKind>;
    assert_eq!(id.0, 42);
  }

  #[test]
  fn token_kind_id_clone() {
    // ensure we don't need to impl Clone for MyKind but the clone is still working
    let id = TokenKindId::new(42) as TokenKindId<MyKind>;
    let id_clone = id.clone();
    assert_eq!(id, id_clone);
  }

  #[test]
  fn token_kind_id_eq() {
    // ensure we don't need to impl PartialEq for MyKind but the eq is still working
    let id1 = TokenKindId::new(42) as TokenKindId<MyKind>;
    let id2 = TokenKindId::new(42) as TokenKindId<MyKind>;
    assert_eq!(id1, id2);
  }

  #[test]
  fn token_kind_id_hash() {
    // ensure we don't need to impl Hash for MyKind but the hash is still working
    let id = TokenKindId::new(42) as TokenKindId<MyKind>;
    let set = HashSet::from([id]);
    assert!(set.contains(&id));
  }
}
