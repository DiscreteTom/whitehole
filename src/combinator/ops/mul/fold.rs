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
    action::{Action, Input},
    combinator::{bytes, take, Bind, Combinator, Take},
    digest::Digest,
    instant::Instant,
  };
  use std::{ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text = Text, State = (), Heap = (), Value = i32>,
    input: &Text,
    value: i32,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action
        .exec(Input {
          instant: &Instant::new(input),
          state: &mut (),
          heap: &mut ()
        })
        .unwrap()
        .value,
      value
    )
  }

  fn accepter() -> Combinator<Bind<Take, i32>> {
    take(1).bind(1)
  }

  fn rejecter() -> Combinator<impl Action<Text = str, State = (), Heap = (), Value = i32>> {
    accepter().reject(|_| true)
  }

  fn accepter_b() -> Combinator<Bind<bytes::Take, i32>> {
    bytes::take(1).bind(1)
  }

  fn rejecter_b() -> Combinator<impl Action<Text = [u8], State = (), Heap = (), Value = i32>> {
    accepter_b().reject(|_| true)
  }

  fn init() -> i32 {
    0
  }

  fn fold(acc: i32, v: i32) -> i32 {
    acc + v
  }

  #[test]
  fn combinator_mul_usize_fold() {
    // normal
    helper((accepter() * 3).fold(init, fold), "123", 3);
    helper((accepter_b() * 3).fold(init, fold), b"123", 3);

    // repeat for 0 times will accept with init value
    helper((accepter() * 0).fold(init, fold), "123", 0);
    helper((accepter_b() * 0).fold(init, fold), b"123", 0);
    helper((accepter().reject(|_| true) * 0).fold(init, fold), "123", 0);
    helper(
      (accepter_b().reject(|_| true) * 0).fold(init, fold),
      b"123",
      0,
    );
  }

  #[test]
  fn combinator_mul_range_fold() {
    // normal
    helper((accepter() * (2..4)).fold(init, fold), "123", 3);
    helper((accepter_b() * (2..4)).fold(init, fold), b"123", 3);

    // repeat for 0 times will accept with init value
    helper((accepter() * (0..1)).fold(init, fold), "123", 0);
    helper((accepter_b() * (0..1)).fold(init, fold), b"123", 0);
    helper((rejecter() * (0..1)).fold(init, fold), "123", 0);
    helper((rejecter_b() * (0..1)).fold(init, fold), b"123", 0);
  }

  #[test]
  fn combinator_mul_range_from_fold() {
    // normal
    helper((accepter() * (2..)).fold(init, fold), "123", 3);
    helper((accepter_b() * (2..)).fold(init, fold), b"123", 3);

    // repeat for 0 times will accept with init value
    helper((rejecter() * (0..)).fold(init, fold), "123", 0);
    helper((rejecter_b() * (0..)).fold(init, fold), b"123", 0);
  }

  #[test]
  fn combinator_mul_range_full_fold() {
    // normal
    helper((accepter() * (..)).fold(init, fold), "123", 3);
    helper((accepter_b() * (..)).fold(init, fold), b"123", 3);

    // repeat for 0 times will accept with init value
    helper((rejecter() * (..)).fold(init, fold), "123", 0);
    helper((rejecter_b() * (..)).fold(init, fold), b"123", 0);
  }

  #[test]
  fn combinator_mul_range_inclusive_fold() {
    // normal
    helper((accepter() * (2..=3)).fold(init, fold), "123", 3);
    helper((accepter_b() * (2..=3)).fold(init, fold), b"123", 3);

    // repeat for 0 times will accept with init value
    helper((accepter() * (0..=0)).fold(init, fold), "123", 0);
    helper((accepter_b() * (0..=0)).fold(init, fold), b"123", 0);
    helper((rejecter() * (0..=0)).fold(init, fold), "123", 0);
    helper((rejecter_b() * (0..=0)).fold(init, fold), b"123", 0);
  }

  #[test]
  fn combinator_mul_range_to_fold() {
    // normal
    helper((accepter() * (..4)).fold(init, fold), "123", 3);
    helper((accepter_b() * (..4)).fold(init, fold), b"123", 3);

    // repeat for 0 times will accept with init value
    helper((accepter() * (..1)).fold(init, fold), "123", 0);
    helper((accepter_b() * (..1)).fold(init, fold), b"123", 0);
    helper((rejecter() * (..1)).fold(init, fold), "123", 0);
    helper((rejecter_b() * (..1)).fold(init, fold), b"123", 0);
  }

  #[test]
  fn combinator_mul_range_to_inclusive_fold() {
    // normal
    helper((accepter() * (2..=3)).fold(init, fold), "123", 3);
    helper((accepter_b() * (2..=3)).fold(init, fold), b"123", 3);

    // repeat for 0 times will accept with init value
    helper((accepter() * (0..=0)).fold(init, fold), "123", 0);
    helper((accepter_b() * (0..=0)).fold(init, fold), b"123", 0);
    helper((rejecter() * (0..=0)).fold(init, fold), "123", 0);
    helper((rejecter_b() * (0..=0)).fold(init, fold), b"123", 0);
  }
}
