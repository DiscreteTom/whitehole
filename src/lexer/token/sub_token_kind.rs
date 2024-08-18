use super::TokenKindId;

/// Usually we use enum to represent a "token kind".
/// Each variant of the enum is a "sub token kind".
///
/// Every sub token kind should have a unique kind id.
/// Usually we create a struct for each variant and implement this
/// trait for those structs so each of them have a unique id.
///
/// This can be auto implemented by applying [`token_kind`](crate::lexer::token::token_kind)
/// to the token kind enum.
/// # Examples
/// ```
/// use whitehole::lexer::token::{token_kind, SubTokenKind};
///
/// #[token_kind]
/// #[derive(Debug)]
/// enum MyKind { A, B(i32) }
///
/// # fn main() {
/// assert_eq!(A::kind_id(), A::kind_id());
/// assert_ne!(A::kind_id(), B::kind_id());
/// # }
/// ```
pub trait SubTokenKind {
  type TokenKind;
  const VARIANT_INDEX: usize;

  /// Return the kind id of this sub token kind.
  #[inline]
  fn kind_id() -> TokenKindId<Self::TokenKind> {
    TokenKindId::new(Self::VARIANT_INDEX)
  }
}

// this is helpful in expectational lexing, if users wants to provide the expected kind id
// they can just use the value (especially for unit variants)
impl<Kind, SubKind: SubTokenKind<TokenKind = Kind>> From<SubKind> for TokenKindId<Kind> {
  #[inline]
  fn from(_: SubKind) -> Self {
    SubKind::kind_id()
  }
}
