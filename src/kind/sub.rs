use super::SubKindId;

/// Usually we use enum to represent a "kind".
/// Each variant of the enum is a "sub kind".
///
/// Every sub kind should have a unique sub kind id.
/// Usually we create a struct for each variant and implement this
/// trait for those structs so each of them have a unique id.
///
/// This can be auto implemented by applying [`whitehole_kind`](crate::kind::whitehole_kind) macro
/// to the kind enum.
/// # Examples
/// ```
/// use whitehole::kind::{whitehole_kind, SubKind};
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

  /// Return the sub kind id of this sub kind.
  #[inline]
  fn kind_id() -> SubKindId<Self::Kind> {
    SubKindId::new(Self::VARIANT_INDEX)
  }
}

// this is helpful in expectational lexing, if users wants to provide the expected sub kind id
// they can just use the value (especially for unit variants)
impl<Kind, Sub: SubKind<Kind = Kind>> From<Sub> for SubKindId<Kind> {
  #[inline]
  fn from(_: Sub) -> Self {
    Sub::kind_id()
  }
}

/// Implement this trait for the kind enum to provide the default sub kind.
///
/// This can be auto implemented by the [`whitehole_kind`](crate::kind::whitehole_kind) macro.
/// # Examples
/// ```
/// use whitehole::kind::{whitehole_kind, KindIdBinding, SubKind, DefaultSubKind};
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
/// users have to `impl Default for SubKindId<MyKind>` manually,
/// but [`Default`] and [`SubKindId`] are both foreign names for user's crate.
///
/// We can't just `impl<T> Default for SubKindId<T>` either
/// because the default sub kind id's value is not always `0`.
pub trait DefaultSubKind: Sized {
  type Default: SubKind<Kind = Self>;

  #[inline]
  fn default_kind_id() -> SubKindId<Self> {
    Self::Default::kind_id()
  }
}
