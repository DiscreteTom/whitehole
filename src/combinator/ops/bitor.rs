//! Overload `|` operator for [`Combinator`].
//!
//! `Combinator | Combinator` will create a new combinator
//! to try to parse with the left-hand side,
//! and if it fails, try to parse with the right-hand side.
//! The new combinator will reject if both of the combinators reject.
//! # Basics
//! ```
//! # use whitehole::{combinator::{eat, Combinator}, action::Action};
//! # fn t(_: Combinator<impl Action>) {}
//! // match "true" or "false"
//! # t(
//! eat("true") | eat("false")
//! # );
//!
//! // you can use a String, a &str, a char or an usize as a shortcut for `eat`
//! // at the right-hand side of `|`
//! # t(
//! eat("true") | "false".to_string()
//! # );
//! # t(
//! eat("true") | "false"
//! # );
//! # t(
//! eat("true") | ';'
//! # );
//! # t(
//! eat("true") | 1
//! # );
//! ```

use crate::combinator::{eat, Action, Combinator, Eat, Input, Output};
use std::ops;

/// An [`Action`] created by the `|` operator.
/// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
#[derive(Debug, Clone, Copy)]
pub struct BitOr<Lhs, Rhs> {
  lhs: Lhs,
  rhs: Rhs,
}

impl<Lhs, Rhs> BitOr<Lhs, Rhs> {
  /// Create a new instance with the left-hand side and right-hand side.
  #[inline]
  const fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self { lhs, rhs }
  }
}

unsafe impl<
    Text: ?Sized,
    State,
    Heap,
    Lhs: Action<Text, State, Heap>,
    Rhs: Action<Text, State, Heap, Value = Lhs::Value>,
  > Action<Text, State, Heap> for BitOr<Lhs, Rhs>
{
  type Value = Lhs::Value;

  #[inline]
  fn exec(&self, mut input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self
      .lhs
      .exec(input.reborrow())
      .or_else(|| self.rhs.exec(input))
  }
}

impl<Lhs, Rhs> ops::BitOr<Combinator<Rhs>> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Rhs>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: Combinator<Rhs>) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, rhs.action))
  }
}

impl<Lhs> ops::BitOr<char> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<char>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: char) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, eat(rhs).action))
  }
}

impl<Lhs> ops::BitOr<usize> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<usize>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: usize) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, eat(rhs).action))
  }
}

impl<Lhs> ops::BitOr<String> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<String>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: String) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, eat(rhs).action))
  }
}

impl<'a, Lhs> ops::BitOr<&'a str> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<&'a str>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: &'a str) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, eat(rhs).action))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{wrap, Input, Output},
    instant::Instant,
  };

  #[test]
  fn combinator_bit_or() {
    let mut state = 0;

    let rejecter = || {
      wrap(|input| {
        *input.state += 1;
        None
      })
    };
    let accepter = || {
      wrap(|input| {
        *input.state += 1;
        input.digest(1)
      })
    };

    // reject then accept, both should increment the state
    assert_eq!(
      (rejecter() | accepter()).exec(Input::new(Instant::new("123"), &mut state, &mut ())),
      Some(Output {
        value: (),
        digested: 1,
      })
    );
    assert_eq!(state, 2);

    state = 0;

    // accept then reject, only the first should increment the state
    assert_eq!(
      (accepter() | rejecter()).exec(Input::new(Instant::new("123"), &mut state, &mut ())),
      Some(Output {
        value: (),
        digested: 1,
      })
    );
    assert_eq!(state, 1);
  }

  #[test]
  fn combinator_bit_or_char() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | '1')
        .exec(Input::new(Instant::new("1"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_usize() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | 1)
        .exec(Input::new(Instant::new("1"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_str() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | "1")
        .exec(Input::new(Instant::new("1"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_string() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | "1".to_string())
        .exec(Input::new(Instant::new("1"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(1)
    );
  }
}
