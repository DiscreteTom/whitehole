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
pub trait SubTokenKind<TokenKindType> {
  /// Return the kind id of this sub token kind.
  fn kind_id() -> &'static TokenKindId<TokenKindType>; // use a static reference to avoid creating a new one every time
}
