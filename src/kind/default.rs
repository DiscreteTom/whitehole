use super::{KindId, SubKind};

/// Implement this trait for the token kind enum to provide the default token kind id.
/// This can be auto implemented by the [`kind`](crate::lexer::token::kind) macro.
/// # Examples
/// ```
/// use whitehole::lexer::token::{
///   kind, KindIdBinding, SubKind, DefaultKindId,
/// };
///
/// #[kind]
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
pub trait DefaultKind: Sized {
  type Default: SubKind<Kind = Self>;

  #[inline]
  fn default_kind_id() -> KindId<Self> {
    Self::Default::kind_id()
  }
}
