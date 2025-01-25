use super::{Mul, Repeat};
use crate::{
  action::{Action, Input, Output},
  combinator::{ops::mul::impl_mul, Combinator},
};
use std::ops;

/// A helper trait to accumulate values when performing `*`
/// on [`Combinator`](crate::combinator::Combinator)s.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
///
/// Built-in implementations are provided for `()`.
pub trait Fold<State = (), Heap = ()> {
  /// The accumulator type.
  type Output: Default;

  /// Fold self with the accumulator.
  fn fold(self, acc: Self::Output, input: Input<&str, &mut State, &mut Heap>) -> Self::Output;
}

impl<State, Heap> Fold<State, Heap> for () {
  type Output = ();
  #[inline]
  fn fold(self, _: Self::Output, _: Input<&str, &mut State, &mut Heap>) -> Self::Output {}
}

impl<Lhs, Rhs: Repeat> ops::Mul<Rhs> for Combinator<Lhs> {
  type Output = Combinator<Mul<Combinator<Lhs>, Rhs>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul::new(self, rhs))
  }
}

unsafe impl<State, Heap, Lhs: Action<State, Heap, Value: Fold<State, Heap>>, Rhs: Repeat>
  Action<State, Heap> for Mul<Combinator<Lhs>, Rhs>
{
  type Value = <Lhs::Value as Fold<State, Heap>>::Output;

  #[inline]
  fn exec(&self, mut input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    impl_mul!(input, self.rhs, Default::default, Fold::fold, self.lhs)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    action::{Action, Input, Output},
    combinator::wrap,
    instant::Instant,
  };

  #[test]
  fn fold_unit() {
    let _: () = ().fold((), Input::new(Instant::new("a"), &mut (), &mut ()));
  }

  #[derive(Debug)]
  struct MyValue(usize);
  impl<State, Heap> Fold<State, Heap> for MyValue {
    type Output = usize;
    fn fold(self, current: Self::Output, _: Input<&str, &mut State, &mut Heap>) -> Self::Output {
      self.0 + current
    }
  }

  #[test]
  fn combinator_mul_usize() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.instant().digested())))
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * 3)
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
      .is_none());

    // repeat rejecter 0 times will accept
    let n = 0;
    assert_eq!(
      (rejecter() * n).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    let n = 0;
    assert_eq!(
      (accepter() * n).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * 3).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // overflow, reject
    assert!((accepter() * 4)
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
      .is_none());
  }

  #[test]
  fn combinator_mul_range() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.instant().digested())))
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..2))
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..2)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..1)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..3)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 1,
        digested: 2
      })
    );

    // too few, reject
    assert!((accepter() * (4..6))
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
      .is_none());
  }

  #[test]
  fn combinator_mul_range_from() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.instant().digested())))
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..))
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // too few, reject
    assert!((accepter() * (4..))
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
      .is_none());
  }

  #[test]
  fn combinator_mul_range_full() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.instant().digested())))
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 3,
        digested: 3
      })
    );
  }

  #[test]
  fn combinator_mul_range_inclusive() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.instant().digested())))
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..=3))
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..=2)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..=0)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..=3)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // too few, reject
    assert!((accepter() * (4..=6))
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
      .is_none());
  }

  #[test]
  fn combinator_mul_range_to() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.instant().digested())))
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..2)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..1)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..3)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 1,
        digested: 2
      })
    );
  }

  #[test]
  fn combinator_mul_range_to_inclusive() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.instant().digested())))
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..=2)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..=0)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..=3)).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 3,
        digested: 3
      })
    );
  }
}
