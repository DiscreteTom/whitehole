use super::{TokenKindId, TokenKindIdProvider};
use std::ops::Deref;

/// Bind the token kind value with an [`TokenKindId`].
/// This is readonly to make sure the binding is not broken.
/// # Examples
/// ```
/// use whitehole_macros::TokenKind;
/// use whitehole::lexer::token::{TokenKindId, TokenKindIdBinding};
///
/// #[derive(TokenKind)]
/// enum MyKind { A, B }
/// // struct `A` and `B` are generated by the macro
/// // and implement `Into<TokenKindIdBinding<MyKind>>`
///
/// let a: TokenKindIdBinding<MyKind> = A.into();
/// let b: TokenKindIdBinding<MyKind> = B.into();
/// assert_eq!(a.id(), &A::kind_id());
/// assert_eq!(b.id(), &B::kind_id()));
/// assert!(matches!(a.value(), MyKind::A));
/// assert!(matches!(b.value(), MyKind::B));
/// ```
#[derive(Debug, Clone)]
pub struct TokenKindIdBinding<TokenKindType> {
  // this is `TokenKindId<TokenKindIdBinding<TokenKindType>>`
  // instead of `TokenKindId<TokenKindType>`
  // because the `kind_id` of generated structs are `TokenKindId<TokenKindIdBinding<TokenKindType>>`
  id: TokenKindId<Self>,
  value: TokenKindType,
}

impl<TokenKindType> TokenKindIdProvider<Self> for TokenKindIdBinding<TokenKindType> {
  fn id(&self) -> &TokenKindId<Self> {
    &self.id
  }
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

  pub fn value(&self) -> &TokenKindType {
    &self.value
  }

  /// Consume self and take the value out.
  pub fn take(self) -> TokenKindType {
    self.value
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, PartialEq)]
  enum MyKind {
    A,
  }

  impl MyKind {
    pub fn f(&self) -> i32 {
      1
    }
  }

  #[test]
  fn token_kind_id_binding() {
    let binding = TokenKindIdBinding::new(42, MyKind::A);
    assert_eq!(binding.id(), &TokenKindId::new(42));
    assert_eq!(binding.value(), &MyKind::A);
    assert_eq!(binding.take(), MyKind::A);
  }

  #[test]
  fn token_kind_id_binding_deref() {
    let binding = TokenKindIdBinding::new(42, MyKind::A);
    assert_eq!(binding.f(), 1);
  }
}
