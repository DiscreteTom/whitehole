//! Overload `|` operator for [`Combinator`].
//!
//! `Combinator | Combinator` will create a new combinator
//! to try to parse with the left-hand side,
//! and if it fails, try to parse with the right-hand side.
//! The new combinator will reject if both of the combinators reject.
//! # Basics
//! ```
//! # use whitehole::{combinator::{eat, Combinator}, action::Action};
//! # fn t(_: Combinator<impl Action<Text = str>>) {}
//! # fn tb(_: Combinator<impl Action<Text = [u8]>>) {}
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

use super::ComposeLiteral;
use crate::{
  action::{Action, Input, Output},
  combinator::{Combinator, Contextual},
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
    Lhs: Action,
    Rhs: Action<Text = Lhs::Text, State = Lhs::State, Heap = Lhs::Heap, Value = Lhs::Value>,
  > Action for BitOr<Lhs, Rhs>
{
  type Text = Lhs::Text;
  type State = Lhs::State;
  type Heap = Lhs::Heap;
  type Value = Lhs::Value;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
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

impl<Lhs: Action<Text: ComposeLiteral<char>>> ops::BitOr<char> for Combinator<Lhs> {
  type Output = Combinator<
    BitOr<Lhs, Contextual<<Lhs::Text as ComposeLiteral<char>>::Output, Lhs::State, Lhs::Heap>>,
  >;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: char) -> Self::Output {
    Self::Output::new(BitOr::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<char>>::to(rhs)),
    ))
  }
}

impl<Lhs: Action<Text: ComposeLiteral<String>>> ops::BitOr<String> for Combinator<Lhs> {
  type Output = Combinator<
    BitOr<Lhs, Contextual<<Lhs::Text as ComposeLiteral<String>>::Output, Lhs::State, Lhs::Heap>>,
  >;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: String) -> Self::Output {
    Self::Output::new(BitOr::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<String>>::to(rhs)),
    ))
  }
}

impl<'a, Lhs: Action<Text: ComposeLiteral<&'a str>>> ops::BitOr<&'a str> for Combinator<Lhs> {
  type Output = Combinator<
    BitOr<Lhs, Contextual<<Lhs::Text as ComposeLiteral<&'a str>>::Output, Lhs::State, Lhs::Heap>>,
  >;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: &'a str) -> Self::Output {
    Self::Output::new(BitOr::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<&'a str>>::to(rhs)),
    ))
  }
}

impl<Lhs: Action<Text: ComposeLiteral<u8>>> ops::BitOr<u8> for Combinator<Lhs> {
  type Output = Combinator<
    BitOr<Lhs, Contextual<<Lhs::Text as ComposeLiteral<u8>>::Output, Lhs::State, Lhs::Heap>>,
  >;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: u8) -> Self::Output {
    Self::Output::new(BitOr::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<u8>>::to(rhs)),
    ))
  }
}

impl<Lhs: Action<Text: ComposeLiteral<Vec<u8>>>> ops::BitOr<Vec<u8>> for Combinator<Lhs> {
  type Output = Combinator<
    BitOr<Lhs, Contextual<<Lhs::Text as ComposeLiteral<Vec<u8>>>::Output, Lhs::State, Lhs::Heap>>,
  >;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: Vec<u8>) -> Self::Output {
    Self::Output::new(BitOr::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<Vec<u8>>>::to(rhs)),
    ))
  }
}

impl<'a, Lhs: Action<Text: ComposeLiteral<&'a [u8]>>> ops::BitOr<&'a [u8]> for Combinator<Lhs> {
  type Output = Combinator<
    BitOr<Lhs, Contextual<<Lhs::Text as ComposeLiteral<&'a [u8]>>::Output, Lhs::State, Lhs::Heap>>,
  >;
  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: &'a [u8]) -> Self::Output {
    Self::Output::new(BitOr::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<&'a [u8]>>::to(rhs)),
    ))
  }
}

impl<'a, const N: usize, Lhs: Action<Text: ComposeLiteral<&'a [u8; N]>>> ops::BitOr<&'a [u8; N]>
  for Combinator<Lhs>
{
  type Output = Combinator<
    BitOr<
      Lhs,
      Contextual<<Lhs::Text as ComposeLiteral<&'a [u8; N]>>::Output, Lhs::State, Lhs::Heap>,
    >,
  >;

  /// See [`ops::bitor`](crate::combinator::ops::bitor) for more information.
  #[inline]
  fn bitor(self, rhs: &'a [u8; N]) -> Self::Output {
    Self::Output::new(BitOr::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<&'a [u8; N]>>::to(rhs)),
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{bytes, wrap, Output},
    contextual,
    digest::Digest,
    instant::Instant,
  };
  use std::{ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest, State>(
    action: impl Action<Text = Text, State = State, Heap = (), Value = ()>,
    input: &Text,
    state: &mut State,
    digested: Option<usize>,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action
        .exec(Input {
          instant: &Instant::new(input),
          state,
          heap: &mut ()
        })
        .map(|o| o.digested),
      digested
    )
  }

  #[test]
  fn combinator_bit_or() {
    contextual!(i32, ());

    let rejecter = || wrap(|_| None).prepare(|input| *input.state += 1);
    let accepter = || wrap(|input| input.instant.accept(1)).prepare(|input| *input.state += 1);

    // reject then accept, both should increment the state
    let mut state = 0;
    helper(rejecter() | accepter(), "123", &mut state, Some(1));
    assert_eq!(state, 2);

    // accept then reject, only the first should increment the state
    let mut state = 0;
    helper(accepter() | rejecter(), "123", &mut state, Some(1));
    assert_eq!(state, 1);
  }

  #[test]
  fn combinator_bit_or_char() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    helper(rejecter() | '1', "1", &mut (), Some(1));
  }

  #[test]
  fn combinator_bit_or_str() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    helper(rejecter() | "1", "1", &mut (), Some(1));
  }

  #[test]
  fn combinator_bit_or_string() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    helper(rejecter() | "1".to_string(), "1", &mut (), Some(1));
  }

  #[test]
  fn combinator_bit_or_u8() {
    let rejecter = || bytes::wrap(|_| Option::<Output<()>>::None);
    helper(rejecter() | b'1', b"1", &mut (), Some(1));
  }

  #[test]
  fn combinator_bit_or_u8_slice() {
    let rejecter = || bytes::wrap(|_| Option::<Output<()>>::None);
    helper(rejecter() | "1".as_bytes(), b"1", &mut (), Some(1));
  }

  #[test]
  fn combinator_bit_or_u8_const_slice() {
    let rejecter = || bytes::wrap(|_| Option::<Output<()>>::None);
    helper(rejecter() | b"1", b"1", &mut (), Some(1));
  }

  #[test]
  fn combinator_bit_or_vec_u8() {
    let rejecter = || bytes::wrap(|_| Option::<Output<()>>::None);
    helper(rejecter() | vec![b'1'], b"1", &mut (), Some(1));
  }
}
