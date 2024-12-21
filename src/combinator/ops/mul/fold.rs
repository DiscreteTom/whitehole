use std::ops;

use crate::{
  action::{Action, Input, Output},
  combinator::{Combinator, EatChar, EatStr, EatString},
};

use super::{impl_mul, impl_mul_with_sep, Mul, Repeat};

/// A helper trait to accumulate values when performing `*`
/// on [`Combinator`](crate::combinator::Combinator)s.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
///
/// Built-in implementations are provided for `()`.
pub trait Fold {
  /// The accumulator type.
  type Output: Default;

  /// Fold self with the accumulator.
  fn fold(self, acc: Self::Output) -> Self::Output;
}

impl Fold for () {
  type Output = ();
  #[inline]
  fn fold(self, _: Self::Output) -> Self::Output {}
}

impl<
    Lhs: Action<Value: Fold>,
    Repeater: Repeat,
    Sep: Action<Value = (), State = Lhs::State, Heap = Lhs::Heap>,
  > ops::Mul<(Repeater, Combinator<Sep>)> for Combinator<Lhs>
{
  type Output = Combinator<Mul<Lhs, (Repeater, Sep)>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: (Repeater, Combinator<Sep>)) -> Self::Output {
    let (range, sep) = rhs;
    Self::Output::new(Mul::new(self.action, (range, sep.action)))
  }
}

impl<Lhs: Action<Value: Fold>, Rhs: Repeat> ops::Mul<Rhs> for Combinator<Lhs> {
  type Output = Combinator<Mul<Lhs, Rhs>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul::new(self.action, rhs))
  }
}

impl<Lhs: Action<Value: Fold>, Rhs: Repeat> Action for Mul<Lhs, Rhs> {
  type Value = <Lhs::Value as Fold>::Output;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Value>> {
    impl_mul(
      &self.lhs,
      &self.rhs,
      Self::Value::default,
      Lhs::Value::fold,
      input,
    )
  }
}

impl<Lhs: Action<Value: Fold>, Repeater: Repeat> ops::Mul<(Repeater, char)> for Combinator<Lhs> {
  type Output = Combinator<Mul<Lhs, (Repeater, EatChar<Lhs::State, Lhs::Heap>)>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: (Repeater, char)) -> Self::Output {
    let (range, sep) = rhs;
    Self::Output::new(Mul::new(self.action, (range, EatChar::new(sep))))
  }
}

impl<Lhs: Action<Value: Fold>, Repeater: Repeat> ops::Mul<(Repeater, String)> for Combinator<Lhs> {
  type Output = Combinator<Mul<Lhs, (Repeater, EatString<Lhs::State, Lhs::Heap>)>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: (Repeater, String)) -> Self::Output {
    let (range, sep) = rhs;
    Self::Output::new(Mul::new(self.action, (range, EatString::new(sep))))
  }
}

impl<'a, Lhs: Action<Value: Fold>, Repeater: Repeat> ops::Mul<(Repeater, &'a str)>
  for Combinator<Lhs>
{
  type Output = Combinator<Mul<Lhs, (Repeater, EatStr<'a, Lhs::State, Lhs::Heap>)>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: (Repeater, &'a str)) -> Self::Output {
    let (range, sep) = rhs;
    Self::Output::new(Mul::new(self.action, (range, EatStr::new(sep))))
  }
}

impl<
    Lhs: Action<Value: Fold>,
    Repeater: Repeat,
    Sep: Action<Value = (), State = Lhs::State, Heap = Lhs::Heap>,
  > Action for Mul<Lhs, (Repeater, Sep)>
{
  type Value = <Lhs::Value as Fold>::Output;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut Lhs::State, &mut Lhs::Heap>,
  ) -> Option<Output<'text, Self::Value>> {
    let (range, sep) = &self.rhs;
    impl_mul_with_sep(
      &self.lhs,
      range,
      sep,
      Self::Value::default,
      Lhs::Value::fold,
      input,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fold_unit() {
    let _: () = ().fold(());
  }
}
