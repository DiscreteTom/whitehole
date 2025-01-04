//! Overload `|` operator for [`Combinator`].
//!
//! `Combinator | Combinator` will create a new combinator
//! to try to parse with the left-hand side,
//! and if it fails, try to parse with the right-hand side.
//! The new combinator will reject if both of the combinators reject.
//! # Basics
//! ```
//! # use whitehole::{combinator::eat, C};
//! # fn t(_: C!()) {}
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

unsafe impl<Lhs: Action, Rhs: Action<Value = Lhs::Value, State = Lhs::State, Heap = Lhs::Heap>>
  Action for BitOr<Lhs, Rhs>
{
  type Value = Lhs::Value;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self
      .lhs
      .exec(input.reborrow())
      .or_else(|| self.rhs.exec(input))
  }
}

impl<Lhs: Action, Rhs: Action<Value = Lhs::Value, State = Lhs::State, Heap = Lhs::Heap>>
  ops::BitOr<Combinator<Rhs>> for Combinator<Lhs>
{
  type Output = Combinator<BitOr<Lhs, Rhs>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: Combinator<Rhs>) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, rhs.action))
  }
}

impl<Lhs: Action> ops::BitOr<char> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<char, Lhs::State, Lhs::Heap>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: char) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, eat(rhs).action))
  }
}

impl<Lhs: Action> ops::BitOr<usize> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<usize, Lhs::State, Lhs::Heap>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: usize) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, eat(rhs).action))
  }
}

impl<Lhs: Action> ops::BitOr<String> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<String, Lhs::State, Lhs::Heap>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: String) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, eat(rhs).action))
  }
}

impl<'a, Lhs: Action> ops::BitOr<&'a str> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<&'a str, Lhs::State, Lhs::Heap>>>;

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
    combinator::{wrap_unchecked, Input, Output},
    instant::Instant,
  };

  #[test]
  fn combinator_bit_or() {
    let mut state = 0;

    let rejecter = || unsafe {
      wrap_unchecked(|input| {
        *input.state += 1;
        None
      })
    };
    let accepter = || unsafe {
      wrap_unchecked(|input| {
        *input.state += 1;
        input.digest(1)
      })
    };

    // reject then accept, both should increment the state
    assert_eq!(
      (rejecter() | accepter()).exec(Input::new(Instant::new("123"), &mut state, &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 1,
      })
    );
    assert_eq!(state, 2);

    state = 0;

    // accept then reject, only the first should increment the state
    assert_eq!(
      (accepter() | rejecter()).exec(Input::new(Instant::new("123"), &mut state, &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 1,
      })
    );
    assert_eq!(state, 1);
  }

  #[test]
  fn combinator_bit_or_char() {
    let rejecter = || unsafe { wrap_unchecked(|_| Option::<Output<()>>::None) };
    assert_eq!(
      (rejecter() | '1')
        .exec(Input::new(Instant::new("1"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_usize() {
    let rejecter = || unsafe { wrap_unchecked(|_| Option::<Output<()>>::None) };
    assert_eq!(
      (rejecter() | 1)
        .exec(Input::new(Instant::new("1"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_str() {
    let rejecter = || unsafe { wrap_unchecked(|_| Option::<Output<()>>::None) };
    assert_eq!(
      (rejecter() | "1")
        .exec(Input::new(Instant::new("1"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_string() {
    let rejecter = || unsafe { wrap_unchecked(|_| Option::<Output<()>>::None) };
    assert_eq!(
      (rejecter() | "1".to_string())
        .exec(Input::new(Instant::new("1"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(1)
    );
  }
}
