use super::TokenKindId;
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
///
/// let a: TokenKindIdBinding<MyKind> = A.into();
/// let b: TokenKindIdBinding<MyKind> = B.into();
/// assert_eq!(a.id(), &TokenKindId::new(0));
/// assert_eq!(b.id(), &TokenKindId::new(1));
/// assert!(matches!(a.value(), MyKind::A));
/// assert!(matches!(b.value(), MyKind::B));
/// ```
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
