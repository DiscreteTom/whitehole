use crate::combinator::Combinator;

/// See [`Combinator::sep`].
#[derive(Debug, Clone, Copy)]
pub struct Sep<T, S> {
  pub(super) value: T,
  pub(super) sep: S,
}

impl<T> Combinator<T> {
  /// Specify an other combinator as the separator
  /// before performing `*` on [`Combinator`]s.
  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  /// # Examples
  /// ```
  /// # use whitehole::{combinator::eat, C};
  /// # fn t(_: C!()) {}
  /// # t(
  /// eat("true").sep(eat(',')) * (1..) // with a combinator
  /// # );
  /// ```
  /// You can use [`char`], `&str`, [`String`], and [`usize`] as the shorthand
  /// for [`eat`](crate::combinator::eat) in the separator.
  /// ```
  /// # use whitehole::{combinator::eat, C};
  /// # fn t(_: C!()) {}
  /// # t(
  /// eat("true").sep(',') * (1..) // with a char
  /// # );
  /// # t(
  /// eat("true").sep(",") * (1..) // with a str
  /// # );
  /// # t(
  /// eat("true").sep(",".to_string()) * (1..) // with a string
  /// # );
  /// # t(
  /// eat("true").sep(1) * (1..) // with a usize
  /// # );
  /// ```
  #[inline]
  pub fn sep<S>(self, sep: impl Into<Combinator<S>>) -> Sep<T, S> {
    Sep {
      value: self.action,
      sep: sep.into().action,
    }
  }
}
