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
//! # fn tb(_: Combinator<impl Action<[u8]>>) {}
//! // match "true" or "false"
//! # t(
//! eat("true") | eat("false")
//! # );
//!
//! // you can use a char, a &str, a String, a u8, a &[u8] or a Vec<u8> as a shortcut for `eat`
//! // at the right-hand side of `|`
//! # t(
//! eat("true") | ';'
//! # );
//! # t(
//! eat("true") | "false"
//! # );
//! # t(
//! eat("true") | "false".to_string()
//! # );
//! # tb(
//! eat(b"true") | b'a'
//! # );
//! # tb(
//! eat(b"true") | b"false"
//! # );
//! # tb(
//! eat(b"true") | b"false".to_vec()
//! # );
//! ```

use crate::{
  action::Context,
  combinator::{eat, Action, Combinator, Eat, Output},
  instant::Instant,
};
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
  fn exec(
    &self,
    instant: &Instant<&Text>,
    mut ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    self
      .lhs
      .exec(&instant, ctx.reborrow())
      .or_else(|| self.rhs.exec(&instant, ctx))
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

impl<'a, Lhs> ops::BitOr<&'a str> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<&'a str>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: &'a str) -> Self::Output {
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

impl<Lhs> ops::BitOr<u8> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<u8>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: u8) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, eat(rhs).action))
  }
}

impl<'a, Lhs> ops::BitOr<&'a [u8]> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<&'a [u8]>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: &'a [u8]) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, eat(rhs).action))
  }
}

impl<'a, const N: usize, Lhs> ops::BitOr<&'a [u8; N]> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<&'a [u8; N]>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: &'a [u8; N]) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, eat(rhs).action))
  }
}

impl<Lhs> ops::BitOr<Vec<u8>> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, Eat<Vec<u8>>>>;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: Vec<u8>) -> Self::Output {
    Self::Output::new(BitOr::new(self.action, eat(rhs).action))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{bytes, wrap, Output},
    instant::Instant,
  };

  #[test]
  fn combinator_bit_or() {
    let mut state = 0;

    let rejecter = || {
      wrap(|_, ctx| {
        *ctx.state += 1;
        None
      })
    };
    let accepter = || {
      wrap(|instant, ctx| {
        *ctx.state += 1;
        instant.accept(1)
      })
    };

    // reject then accept, both should increment the state
    assert_eq!(
      (rejecter() | accepter()).exec(
        &Instant::new("123"),
        Context {
          state: &mut state,
          heap: &mut ()
        }
      ),
      Some(Output {
        value: (),
        digested: 1,
      })
    );
    assert_eq!(state, 2);

    state = 0;

    // accept then reject, only the first should increment the state
    assert_eq!(
      (accepter() | rejecter()).exec(
        &Instant::new("123"),
        Context {
          state: &mut state,
          heap: &mut ()
        }
      ),
      Some(Output {
        value: (),
        digested: 1,
      })
    );
    assert_eq!(state, 1);
  }

  #[test]
  fn combinator_bit_or_char() {
    let rejecter = || wrap(|_, _| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | '1')
        .exec(&Instant::new("1"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_str() {
    let rejecter = || wrap(|_, _| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | "1")
        .exec(&Instant::new("1"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_string() {
    let rejecter = || wrap(|_, _| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | "1".to_string())
        .exec(&Instant::new("1"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_u8() {
    let rejecter = || bytes::wrap(|_, _| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | b'1')
        .exec(&Instant::new(b"1"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_u8_slice() {
    let rejecter = || bytes::wrap(|_, _| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | "1".as_bytes())
        .exec(&Instant::new(b"1"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_u8_const_slice() {
    let rejecter = || bytes::wrap(|_, _| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | b"1")
        .exec(&Instant::new(b"1"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
  }

  #[test]
  fn combinator_bit_or_vec_u8() {
    let rejecter = || bytes::wrap(|_, _| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | vec![b'1'])
        .exec(&Instant::new(b"1"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
  }
}
