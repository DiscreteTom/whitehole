use super::{inline::InlineFold, Fold, Mul, Repeat};
use crate::{
  action::{shift_input, Action, Input, Output},
  combinator::{ops::mul::impl_mul_with_sep, Combinator},
};

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
  /// # use whitehole::{combinator::{eat, Combinator}, action::Action};
  /// # fn t(_: Combinator<impl Action>) {}
  /// # t(
  /// eat("true").sep(eat(',')) * (1..) // with a combinator
  /// # );
  /// ```
  /// You can use [`char`], `&str`, [`String`], and [`usize`] as the shorthand
  /// for [`eat`](crate::combinator::eat) in the separator.
  /// ```
  /// # use whitehole::{combinator::{eat, Combinator}, action::Action};
  /// # fn t(_: Combinator<impl Action>) {}
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
  pub fn sep<S>(self, sep: impl Into<Combinator<S>>) -> Combinator<Sep<T, S>> {
    Combinator::new(Sep {
      value: self.action,
      sep: sep.into().action,
    })
  }
}

unsafe impl<
    Lhs: Action<State, Heap, Value: Fold<State, Heap>>,
    Rhs: Repeat,
    S: Action<State, Heap>,
    State,
    Heap,
  > Action<State, Heap> for Sep<Mul<Combinator<Lhs>, Rhs>, S>
{
  type Value = <Lhs::Value as Fold<State, Heap>>::Output;

  fn exec(&self, mut input: Input<&mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    impl_mul_with_sep!(
      input,
      self.value.rhs,
      Default::default,
      Fold::fold,
      self.value.lhs,
      self.sep
    )
  }
}

unsafe impl<
    T: Action<State, Heap>,
    Acc,
    Repeater: Repeat,
    Init: Fn() -> Acc,
    Folder: Fn(T::Value, Acc, Input<&mut State, &mut Heap>) -> Acc,
    S: Action<State, Heap>,
    State,
    Heap,
  > Action<State, Heap> for Sep<Mul<InlineFold<T, Init, Folder>, Repeater>, S>
{
  type Value = Acc;

  fn exec(&self, mut input: Input<&mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    impl_mul_with_sep!(
      input,
      self.value.rhs,
      self.value.lhs.init,
      self.value.lhs.fold,
      self.value.lhs.action,
      self.sep
    )
  }
}
