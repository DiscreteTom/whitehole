use super::Mul;
use crate::combinator::Combinator;

impl<Lhs, Rhs, Sep, Init, Fold> Combinator<Mul<Lhs, Rhs, Sep, Init, Fold>> {
  /// Fold values with an ad-hoc accumulator.
  ///
  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  /// # Examples
  /// ```
  /// # use whitehole::{combinator::next, action::{Input, Action}, instant::Instant};
  /// let combinator = {
  ///   // accept one ascii digit at a time
  ///   next(|c| c.is_ascii_digit())
  ///     // convert the char to a number
  ///     .select(|ctx| ctx.input().instant().rest().chars().next().unwrap() as usize - '0' as usize)
  ///     // repeat for 1 or more times
  ///     * (1..)
  /// }
  /// // init accumulator with 0, and fold values
  /// .fold(|| 0 as usize, |value, acc, _| acc * 10 + value)
  ///
  /// // parse "123" to 123
  /// assert_eq!(
  ///   combinator.exec(Input::new(Instant::new("123"), &mut (), &mut ())).unwrap().value,
  ///   123
  /// )
  /// ```
  #[inline]
  pub fn fold<Value, Acc, NewInit: Fn() -> Acc, NewFold: Fn(Value, Acc) -> Acc>(
    self,
    init: NewInit,
    fold: NewFold,
  ) -> Combinator<Mul<Lhs, Rhs, Sep, NewInit, NewFold>> {
    Combinator::new(Mul {
      lhs: self.action.lhs,
      rhs: self.action.rhs,
      sep: self.action.sep,
      init,
      fold,
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    action::{Action, Input, Output},
    combinator::wrap,
    instant::Instant,
  };

  #[test]
  fn combinator_mul_usize() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| input.instant().digested()))
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
      (accepter() * n)
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * 3)
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // overflow, reject
    assert!((accepter() * 4)
      .fold(|| 0, |v, acc| acc + v)
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
          .map(|output| output.map(|_| input.instant().digested()))
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
      (accepter() * (0..1))
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..3))
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 1,
        digested: 2
      })
    );

    // too few, reject
    assert!((accepter() * (4..6))
      .fold(|| 0, |v, acc| acc + v)
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
          .map(|output| output.map(|_| input.instant().digested()))
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
      (accepter() * (0..))
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // too few, reject
    assert!((accepter() * (4..))
      .fold(|| 0, |v, acc| acc + v)
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
          .map(|output| output.map(|_| input.instant().digested()))
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
      (accepter() * (..))
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
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
          .map(|output| output.map(|_| input.instant().digested()))
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
      (accepter() * (0..=0))
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..=3))
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // too few, reject
    assert!((accepter() * (4..=6))
      .fold(|| 0, |v, acc| acc + v)
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
          .map(|output| output.map(|_| input.instant().digested()))
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
      (accepter() * (..1))
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..3))
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
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
          .map(|output| output.map(|_| input.instant().digested()))
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
      (accepter() * (..=0))
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..=3))
        .fold(|| 0, |v, acc| acc + v)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 3,
        digested: 3
      })
    );
  }
}
