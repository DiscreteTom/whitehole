use super::{Mul, Repeat};
use crate::{action::Action, combinator::Combinator};
use std::ops;

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

impl<Lhs: Action<Value: Fold>, Rhs: Repeat> ops::Mul<Rhs> for Combinator<Lhs> {
  type Output = Combinator<
    Mul<
      Lhs,
      (
        Rhs,
        fn() -> <Lhs::Value as Fold>::Output,
        fn(Lhs::Value, <Lhs::Value as Fold>::Output) -> <Lhs::Value as Fold>::Output,
      ),
    >,
  >;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul::new(self.action, (rhs, Default::default, Fold::fold)))
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
