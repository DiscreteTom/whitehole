use super::{Mul, Repeat, Sep};
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

impl<T: Action<Value: Fold>, S: Action<State = T::State, Heap = T::Heap>, Rhs: Repeat> ops::Mul<Rhs>
  for Sep<T, S>
{
  type Output = Combinator<
    Mul<
      Sep<T, S>,
      (
        Rhs,
        fn() -> <T::Value as Fold>::Output,
        fn(T::Value, <T::Value as Fold>::Output) -> <T::Value as Fold>::Output,
      ),
    >,
  >;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul::new(self, (rhs, Default::default, Fold::fold)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    action::{Action, Input, Output},
    combinator::eat,
  };

  #[test]
  fn fold_unit() {
    let _: () = ().fold(());
  }

  #[test]
  fn combinator_mul_with_sep() {
    let one_or_more = || eat('a').sep(',') * (1..);
    macro_rules! input {
      ($rest:expr) => {
        Input::new($rest, 0, &mut (), &mut ()).unwrap()
      };
    }

    assert_eq!(one_or_more().exec(input!(",")), None);
    assert_eq!(
      one_or_more().exec(input!("a")),
      Some(Output {
        value: (),
        digested: 1
      })
    );
    assert_eq!(
      one_or_more().exec(input!("a,")),
      Some(Output {
        value: (),
        digested: 1
      })
    );
    assert_eq!(
      one_or_more().exec(input!("a,a")),
      Some(Output {
        value: (),
        digested: 3
      })
    );
    assert_eq!(
      one_or_more().exec(input!("a,,")),
      Some(Output {
        value: (),
        digested: 1
      })
    );
    assert_eq!(
      one_or_more().exec(input!("a,aa")),
      Some(Output {
        value: (),
        digested: 3
      })
    );
  }
}
