use super::KindId;

/// Usually we use enum to represent a "token kind".
/// Each variant of the enum is a "sub token kind".
///
/// Every sub token kind should have a unique kind id.
/// Usually we create a struct for each variant and implement this
/// trait for those structs so each of them have a unique id.
///
/// This can be auto implemented by applying [`kind`](crate::lexer::token::kind)
/// to the token kind enum.
/// # Examples
/// ```
/// use whitehole::lexer::token::{kind, SubKind};
///
/// #[kind]
/// #[derive(Debug)]
/// enum MyKind { A, B(i32) }
///
/// # fn main() {
/// assert_eq!(A::kind_id(), A::kind_id());
/// assert_ne!(A::kind_id(), B::kind_id());
/// # }
/// ```
pub trait SubKind {
  type Kind;
  const VARIANT_INDEX: usize;

  /// Return the kind id of this sub token kind.
  #[inline]
  fn kind_id() -> KindId<Self::Kind> {
    KindId::new(Self::VARIANT_INDEX)
  }
}

// this is helpful in expectational lexing, if users wants to provide the expected kind id
// they can just use the value (especially for unit variants)
impl<Kind, Sub: SubKind<Kind = Kind>> From<Sub> for KindId<Kind> {
  #[inline]
  fn from(_: Sub) -> Self {
    Sub::kind_id()
  }
}
