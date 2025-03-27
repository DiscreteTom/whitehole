use super::Mul;
use crate::combinator::Combinator;

impl<Lhs, Rhs, Sep, Init, Fold> Combinator<Mul<Lhs, Rhs, Sep, Init, Fold>> {
  /// Fold values with an ad-hoc accumulator.
  ///
  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  /// # Examples
  /// ```
  /// # use whitehole::{combinator::next, parser::Parser};
  /// let entry = {
  ///   // accept one ascii digit at a time
  ///   next(|c| c.is_ascii_digit())
  ///     // convert the char to a number
  ///     .select(|accept, _| accept.instant().rest().chars().next().unwrap() as usize - '0' as usize)
  ///     // repeat for 1 or more times
  ///     * (1..)
  /// }
  /// // init accumulator with 0, and fold values
  /// .fold(|| 0 as usize, |acc, value| acc * 10 + value);
  ///
  /// // parse "123" to 123
  /// assert_eq!(
  ///   Parser::builder().entry(entry).build("123").next().unwrap().value,
  ///   123
  /// )
  /// ```
  #[inline]
  pub fn fold<Value, Acc, NewInit: Fn() -> Acc, NewFold: Fn(Acc, Value) -> Acc>(
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
    digest::Digest,
    instant::Instant,
  };
  use std::{fmt::Debug, ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest, Value: PartialEq + Debug>(
    action: impl Action<Text, State = (), Heap = (), Value = Value>,
    input: &Text,
    expected: Option<Output<Value>>,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action.exec(Input {
        instant: &Instant::new(input),
        state: &mut (),
        heap: &mut ()
      }),
      expected
    )
  }

  #[test]
  fn combinator_mul_usize() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|instant| {
        instant
          .accept(1)
          .map(|output| output.map(|_| instant.digested()))
      })
    };

    // repeat a rejecter will reject
    helper(rejecter() * 3, "123", None);

    // repeat rejecter 0 times will accept
    let n = 0;
    helper(
      rejecter() * n,
      "123",
      Some(Output {
        value: (),
        digested: 0,
      }),
    );

    // repeat an accepter 0 times will accept
    let n = 0;
    helper(
      (accepter() * n).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 0,
        digested: 0,
      }),
    );

    // normal, apply the folded value and sum the digested
    helper(
      (accepter() * 3).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 3,
        digested: 3,
      }),
    );

    // overflow, reject
    helper((accepter() * 4).fold(|| 0, |acc, v| acc + v), "123", None);
  }

  #[test]
  fn combinator_mul_range() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|instant| {
        instant
          .accept(1)
          .map(|output| output.map(|_| instant.digested()))
      })
    };

    // repeat a rejecter will reject
    helper(rejecter() * (1..2), "123", None);

    // repeat rejecter 0 times will accept
    helper(
      rejecter() * (0..2),
      "123",
      Some(Output {
        value: (),
        digested: 0,
      }),
    );

    // repeat an accepter 0 times will accept
    helper(
      (accepter() * (0..1)).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 0,
        digested: 0,
      }),
    );

    // normal, apply the folded value and sum the digested
    helper(
      (accepter() * (0..3)).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 1,
        digested: 2,
      }),
    );

    // too few, reject
    helper(
      (accepter() * (4..6)).fold(|| 0, |acc, v| acc + v),
      "123",
      None,
    );
  }

  #[test]
  fn combinator_mul_range_from() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|instant| {
        instant
          .accept(1)
          .map(|output| output.map(|_| instant.digested()))
      })
    };

    // repeat a rejecter will reject
    helper(rejecter() * (1..), "123", None);

    // repeat rejecter 0 times will accept
    helper(
      rejecter() * (0..),
      "123",
      Some(Output {
        value: (),
        digested: 0,
      }),
    );

    // normal, apply the folded value and sum the digested
    helper(
      (accepter() * (0..)).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 3,
        digested: 3,
      }),
    );

    // too few, reject
    helper(
      (accepter() * (4..)).fold(|| 0, |acc, v| acc + v),
      "123",
      None,
    );
  }

  #[test]
  fn combinator_mul_range_full() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|instant| {
        instant
          .accept(1)
          .map(|output| output.map(|_| instant.digested()))
      })
    };

    // repeat rejecter 0 times will accept
    helper(
      rejecter() * (..),
      "123",
      Some(Output {
        value: (),
        digested: 0,
      }),
    );

    // normal, apply the folded value and sum the digested
    helper(
      (accepter() * (..)).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 3,
        digested: 3,
      }),
    );
  }

  #[test]
  fn combinator_mul_range_inclusive() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|instant| {
        instant
          .accept(1)
          .map(|output| output.map(|_| instant.digested()))
      })
    };

    // repeat a rejecter will reject
    helper(rejecter() * (1..=3), "123", None);

    // repeat rejecter 0 times will accept
    helper(
      rejecter() * (0..=2),
      "123",
      Some(Output {
        value: (),
        digested: 0,
      }),
    );

    // repeat an accepter 0 times will accept
    helper(
      (accepter() * (0..=0)).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 0,
        digested: 0,
      }),
    );

    // normal, apply the folded value and sum the digested
    helper(
      (accepter() * (0..=3)).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 3,
        digested: 3,
      }),
    );

    // too few, reject
    helper(
      (accepter() * (4..=6)).fold(|| 0, |acc, v| acc + v),
      "123",
      None,
    );
  }

  #[test]
  fn combinator_mul_range_to() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|instant| {
        instant
          .accept(1)
          .map(|output| output.map(|_| instant.digested()))
      })
    };

    // repeat rejecter 0 times will accept
    helper(
      rejecter() * (..2),
      "123",
      Some(Output {
        value: (),
        digested: 0,
      }),
    );

    // repeat an accepter 0 times will accept
    helper(
      (accepter() * (..1)).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 0,
        digested: 0,
      }),
    );

    // normal, apply the folded value and sum the digested
    helper(
      (accepter() * (..3)).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 1,
        digested: 2,
      }),
    );
  }

  #[test]
  fn combinator_mul_range_to_inclusive() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|instant| {
        instant
          .accept(1)
          .map(|output| output.map(|_| instant.digested()))
      })
    };

    // repeat rejecter 0 times will accept
    helper(
      rejecter() * (..=2),
      "123",
      Some(Output {
        value: (),
        digested: 0,
      }),
    );

    // repeat an accepter 0 times will accept
    helper(
      (accepter() * (..=0)).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 0,
        digested: 0,
      }),
    );

    // normal, apply the folded value and sum the digested
    helper(
      (accepter() * (..=3)).fold(|| 0, |acc, v| acc + v),
      "123",
      Some(Output {
        value: 3,
        digested: 3,
      }),
    );
  }
}
