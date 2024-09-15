use super::SubKindId;

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
/// #[whitehole_kind]
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
  fn kind_id() -> SubKindId<Self::Kind> {
    SubKindId::new(Self::VARIANT_INDEX)
  }
}

// this is helpful in expectational lexing, if users wants to provide the expected kind id
// they can just use the value (especially for unit variants)
impl<Kind, Sub: SubKind<Kind = Kind>> From<Sub> for SubKindId<Kind> {
  #[inline]
  fn from(_: Sub) -> Self {
    Sub::kind_id()
  }
}

/// Implement this trait for the token kind enum to provide the default token kind id.
/// This can be auto implemented by the [`kind`](crate::lexer::token::kind) macro.
/// # Examples
/// ```
/// use whitehole::lexer::token::{
///   kind, KindIdBinding, SubKind, DefaultKindId,
/// };
///
/// #[whitehole_kind]
/// #[derive(Default, Debug, PartialEq, Eq)]
/// enum MyKind {
///   #[default]
///   A
/// }
///
/// # fn main() {
/// assert_eq!(MyKind::default_kind_id(), A::kind_id());
/// assert_eq!(MyKind::default(), MyKind::A);
/// # }
/// ```
/// # Design
/// We can't replace this with [`Default`] because otherwise
/// users have to `impl Default for KindId<MyKind>` manually,
/// but [`Default`] and [`KindId`] are both foreign names for user's crate.
///
/// We can't just `impl<T> Default for KindId<T>` either
/// because the default token kind id's value is not always `0`.
pub trait DefaultSubKind: Sized {
  type Default: SubKind<Kind = Self>;

  #[inline]
  fn default_kind_id() -> SubKindId<Self> {
    Self::Default::kind_id()
  }
}
