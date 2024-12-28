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
  #[inline]
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
  #[inline]
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul::new(self, (rhs, Default::default, Fold::fold)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    action::{Action, Input, Output},
    combinator::{eat, wrap},
  };

  #[test]
  fn fold_unit() {
    let _: () = ().fold(());
  }

  #[derive(Debug)]
  struct MyValue(usize);
  impl Fold for MyValue {
    type Output = usize;
    fn fold(self, current: Self::Output) -> Self::Output {
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
          .map(|output| output.map(|_| MyValue(input.start())))
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * 3)
      .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    let n = 0;
    assert_eq!(
      (rejecter() * n).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    let n = 0;
    assert_eq!(
      (accepter() * n).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * 3).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // overflow, reject
    assert!((accepter() * 4)
      .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.start())))
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..2))
      .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..2)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..1)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..3)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 1,
        digested: 2
      })
    );

    // too few, reject
    assert!((accepter() * (4..6))
      .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_from() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.start())))
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..))
      .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // too few, reject
    assert!((accepter() * (4..))
      .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_full() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.start())))
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
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
          .map(|output| output.map(|_| MyValue(input.start())))
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..=3))
      .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..=2)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..=0)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..=3)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // too few, reject
    assert!((accepter() * (4..=6))
      .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_to() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.start())))
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..2)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..1)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..3)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
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
          .map(|output| output.map(|_| MyValue(input.start())))
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..=2)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..=0)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..=3)).exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 3,
        digested: 3
      })
    );
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
