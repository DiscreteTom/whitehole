//! Overload `+` operator for [`Combinator`].
//!
//! `Combinator + Combinator` will create a new combinator
//! to parse with the left-hand side, then parse with the right-hand side.
//! The combinator will return the output with [`Concat`]-ed value
//! and the rest of the input text after the right-hand side is executed,
//! or reject if any of the combinators rejects.
//! # Basics
//! ```
//! # use whitehole::{combinator::{eat, Combinator}, action::Action};
//! # fn t(_: Combinator<impl Action>) {}
//! # fn tb(_: Combinator<impl Action<[u8]>>) {}
//! // match "123" then match "456"
//! # t(
//! eat("123") + eat("456")
//! # );
//!
//! // you can use a char, a &str, a String, a u8, a &[u8] or a Vec<u8> as a shortcut for `eat`
//! // at the right-hand side of `+`
//! # t(
//! eat("true") + ';'
//! # );
//! # t(
//! eat("true") + "false"
//! # );
//! # t(
//! eat("true") + "false".to_string()
//! # );
//! # tb(
//! eat(b"true") + b'a'
//! # );
//! # tb(
//! eat(b"true") + b"false"
//! # );
//! # tb(
//! eat(b"true") + b"false".to_vec()
//! # );
//! ```
//! # Concat Values
//! If your combinators' values are tuples, they can be concatenated,
//! and all unit tuples will be ignored.
//! ```
//! # use whitehole::{combinator::{next, eat}, action::Action, parser::Parser};
//! let integer = || {
//!   (next(|c| c.is_ascii_digit()) * (1..)) // eat one or more digits
//!     .select(|accept, _| accept.content().parse::<usize>().unwrap()) // parse the digits
//!     .tuple() // wrap the parsed digits in a tuple
//! };
//! let dot = eat('.'); // the value is `()`
//!
//! // (usize,) + () + (usize,)
//! // unit tuples will be ignored
//! // so the value will be `(usize, usize)`
//! let entry = integer() + dot + integer();
//!
//! let mut parser = Parser::builder().entry(entry).build("123.456");
//! let output = parser.next().unwrap();
//! assert_eq!(output.value, (123, 456));
//! ```
//! See [`Concat`] for more information.

mod concat;

pub use concat::*;

use crate::{
  action::Context,
  combinator::{eat, Action, Combinator, Eat, Output},
  digest::Digest,
  instant::Instant,
};
use std::{
  ops::{self, RangeFrom},
  slice::SliceIndex,
};

/// An [`Action`] created by the `+` operator.
/// See [`ops::add`](crate::combinator::ops::add) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Add<Lhs, Rhs> {
  lhs: Lhs,
  rhs: Rhs,
}

impl<Lhs, Rhs> Add<Lhs, Rhs> {
  /// Create a new instance with the left-hand side and right-hand side.
  #[inline]
  const fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self { lhs, rhs }
  }
}

unsafe impl<
    Text: ?Sized + Digest,
    State,
    Heap,
    Lhs: Action<Text, State, Heap, Value: Concat<Rhs::Value>>,
    Rhs: Action<Text, State, Heap>,
  > Action<Text, State, Heap> for Add<Lhs, Rhs>
where
  RangeFrom<usize>: SliceIndex<Text, Output = Text>,
{
  type Value = <Lhs::Value as Concat<Rhs::Value>>::Output;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    mut ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    self.lhs.exec(instant, ctx.reborrow()).and_then(|output| {
      self
        .rhs
        .exec(&unsafe { instant.shift_unchecked(output.digested) }, ctx)
        .map(|rhs_output| Output {
          value: output.value.concat(rhs_output.value),
          digested: unsafe { output.digested.unchecked_add(rhs_output.digested) },
        })
    })
  }
}

impl<Lhs, Rhs> ops::Add<Combinator<Rhs>> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Rhs>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: Combinator<Rhs>) -> Self::Output {
    Self::Output::new(Add::new(self.action, rhs.action))
  }
}

impl<Lhs> ops::Add<char> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Eat<char>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: char) -> Self::Output {
    Self::Output::new(Add::new(self.action, eat(rhs).action))
  }
}

impl<'a, Lhs> ops::Add<&'a str> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Eat<&'a str>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a str) -> Self::Output {
    Self::Output::new(Add::new(self.action, eat(rhs).action))
  }
}

impl<Lhs> ops::Add<String> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Eat<String>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: String) -> Self::Output {
    Self::Output::new(Add::new(self.action, eat(rhs).action))
  }
}

impl<Lhs> ops::Add<u8> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Eat<u8>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: u8) -> Self::Output {
    Self::Output::new(Add::new(self.action, eat(rhs).action))
  }
}

impl<'a, Lhs> ops::Add<&'a [u8]> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Eat<&'a [u8]>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a [u8]) -> Self::Output {
    Self::Output::new(Add::new(self.action, eat(rhs).action))
  }
}

impl<'a, const N: usize, Lhs> ops::Add<&'a [u8; N]> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Eat<&'a [u8; N]>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a [u8; N]) -> Self::Output {
    Self::Output::new(Add::new(self.action, eat(rhs).action))
  }
}

impl<Lhs> ops::Add<Vec<u8>> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Eat<Vec<u8>>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: Vec<u8>) -> Self::Output {
    Self::Output::new(Add::new(self.action, eat(rhs).action))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{bytes, wrap},
    instant::Instant,
  };

  #[test]
  fn combinator_add() {
    let rejecter = || wrap(|_, _| Option::<Output<()>>::None);
    let accepter_unit = || wrap(|instant, _| instant.accept(1));
    let accepter_int = || wrap(|instant, _| instant.accept(1).map(|output| output.map(|_| (123,))));

    // reject then accept, should return None
    assert!((rejecter() + accepter_unit())
      .exec(&Instant::new("123"), Context::default())
      .is_none());

    // accept then reject, should return None
    assert!((accepter_unit() + rejecter())
      .exec(&Instant::new("123"), Context::default())
      .is_none());

    // accept then accept, should return the sum of the digested
    // with the concat value
    assert_eq!(
      (accepter_unit() + accepter_int()).exec(&Instant::new("123"), Context::default()),
      Some(Output {
        value: (123,),
        digested: 2,
      })
    );
    assert_eq!(
      (accepter_int() + accepter_unit()).exec(&Instant::new("123"), Context::default()),
      Some(Output {
        value: (123,),
        digested: 2,
      })
    );
    assert_eq!(
      (accepter_int() + accepter_int()).exec(&Instant::new("123"), Context::default()),
      Some(Output {
        value: (123, 123),
        digested: 2,
      })
    );
  }

  #[test]
  fn combinator_add_char() {
    let eat1 = || wrap(|instant, _| instant.accept(1));

    assert_eq!(
      (eat1() + '2')
        .exec(&Instant::new("12"), Context::default())
        .map(|output| output.digested),
      Some(2)
    );
  }

  #[test]
  fn combinator_add_str() {
    let eat1 = || wrap(|instant, _| instant.accept(1));

    assert_eq!(
      (eat1() + "23")
        .exec(&Instant::new("123"), Context::default())
        .map(|output| output.digested),
      Some(3)
    );
  }

  #[test]
  fn combinator_add_string() {
    let eat1 = || wrap(|instant, _| instant.accept(1));

    assert_eq!(
      (eat1() + "23".to_string())
        .exec(&Instant::new("123"), Context::default())
        .map(|output| output.digested),
      Some(3)
    );
  }

  #[test]
  fn combinator_add_u8() {
    let eat1 = || bytes::wrap(|instant, _| instant.accept(1));

    assert_eq!(
      (eat1() + b'2')
        .exec(&Instant::new(b"123"), Context::default())
        .map(|output| output.digested),
      Some(2)
    );
  }

  #[test]
  fn combinator_add_u8_slice() {
    let eat1 = || bytes::wrap(|instant, _| instant.accept(1));

    assert_eq!(
      (eat1() + "2".as_bytes())
        .exec(&Instant::new(b"123"), Context::default())
        .map(|output| output.digested),
      Some(2)
    );
  }

  #[test]
  fn combinator_add_u8_const_slice() {
    let eat1 = || bytes::wrap(|instant, _| instant.accept(1));

    assert_eq!(
      (eat1() + b"2")
        .exec(&Instant::new(b"123"), Context::default())
        .map(|output| output.digested),
      Some(2)
    );
  }

  #[test]
  fn combinator_add_vec_u8() {
    let eat1 = || bytes::wrap(|instant, _| instant.accept(1));

    assert_eq!(
      (eat1() + vec![b'2'])
        .exec(&Instant::new(b"123"), Context::default())
        .map(|output| output.digested),
      Some(2)
    );
  }
}
