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
//! # fn t(_: Combinator<impl Action<Text = str>>) {}
//! # fn tb(_: Combinator<impl Action<Text = [u8]>>) {}
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
  action::{Action, Input, Output},
  combinator::{bytes, Combinator, Contextual, Eat},
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
    Lhs: Action<Text: Digest, Value: Concat<Rhs::Value>>,
    Rhs: Action<Text = Lhs::Text, State = Lhs::State, Heap = Lhs::Heap>,
  > Action for Add<Lhs, Rhs>
where
  RangeFrom<usize>: SliceIndex<Lhs::Text, Output = Lhs::Text>,
{
  type Text = Lhs::Text;
  type State = Lhs::State;
  type Heap = Lhs::Heap;
  type Value = <Lhs::Value as Concat<Rhs::Value>>::Output;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.lhs.exec(input.reborrow()).and_then(|output| {
      self
        .rhs
        .exec(input.reborrow_with(&unsafe { input.instant.to_digested_unchecked(output.digested) }))
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

// TODO: comments, move to a better place
pub trait ComposeLiteral<Rhs> {
  type Output;

  fn to(rhs: Rhs) -> Self::Output;
}

impl ComposeLiteral<char> for str {
  type Output = Eat<char>;

  #[inline]
  fn to(rhs: char) -> Self::Output {
    Eat::new(rhs)
  }
}

impl ComposeLiteral<String> for str {
  type Output = Eat<String>;

  #[inline]
  fn to(rhs: String) -> Self::Output {
    Eat::new(rhs)
  }
}

impl<'a> ComposeLiteral<&'a str> for str {
  type Output = Eat<&'a str>;

  #[inline]
  fn to(rhs: &'a str) -> Self::Output {
    Eat::new(rhs)
  }
}

impl ComposeLiteral<u8> for [u8] {
  type Output = bytes::Eat<u8>;

  #[inline]
  fn to(rhs: u8) -> Self::Output {
    bytes::Eat::new(rhs)
  }
}

impl ComposeLiteral<Vec<u8>> for [u8] {
  type Output = bytes::Eat<Vec<u8>>;

  #[inline]
  fn to(rhs: Vec<u8>) -> Self::Output {
    bytes::Eat::new(rhs)
  }
}

impl<'a> ComposeLiteral<&'a [u8]> for [u8] {
  type Output = bytes::Eat<&'a [u8]>;

  #[inline]
  fn to(rhs: &'a [u8]) -> Self::Output {
    bytes::Eat::new(rhs)
  }
}

impl<'a, const N: usize> ComposeLiteral<&'a [u8; N]> for [u8] {
  type Output = bytes::Eat<&'a [u8; N]>;

  #[inline]
  fn to(rhs: &'a [u8; N]) -> Self::Output {
    bytes::Eat::new(rhs)
  }
}

impl<Lhs: Action<Text: ComposeLiteral<char>>> ops::Add<char> for Combinator<Lhs> {
  type Output = Combinator<
    Add<Lhs, Contextual<<Lhs::Text as ComposeLiteral<char>>::Output, Lhs::State, Lhs::Heap>>,
  >;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: char) -> Self::Output {
    Self::Output::new(Add::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<char>>::to(rhs)),
    ))
  }
}

impl<Lhs: Action<Text: ComposeLiteral<String>>> ops::Add<String> for Combinator<Lhs> {
  type Output = Combinator<
    Add<Lhs, Contextual<<Lhs::Text as ComposeLiteral<String>>::Output, Lhs::State, Lhs::Heap>>,
  >;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: String) -> Self::Output {
    Self::Output::new(Add::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<String>>::to(rhs)),
    ))
  }
}

impl<'a, Lhs: Action<Text: ComposeLiteral<&'a str>>> ops::Add<&'a str> for Combinator<Lhs> {
  type Output = Combinator<
    Add<Lhs, Contextual<<Lhs::Text as ComposeLiteral<&'a str>>::Output, Lhs::State, Lhs::Heap>>,
  >;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a str) -> Self::Output {
    Self::Output::new(Add::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<&'a str>>::to(rhs)),
    ))
  }
}

impl<Lhs: Action<Text: ComposeLiteral<u8>>> ops::Add<u8> for Combinator<Lhs> {
  type Output = Combinator<
    Add<Lhs, Contextual<<Lhs::Text as ComposeLiteral<u8>>::Output, Lhs::State, Lhs::Heap>>,
  >;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: u8) -> Self::Output {
    Self::Output::new(Add::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<u8>>::to(rhs)),
    ))
  }
}

impl<Lhs: Action<Text: ComposeLiteral<Vec<u8>>>> ops::Add<Vec<u8>> for Combinator<Lhs> {
  type Output = Combinator<
    Add<Lhs, Contextual<<Lhs::Text as ComposeLiteral<Vec<u8>>>::Output, Lhs::State, Lhs::Heap>>,
  >;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: Vec<u8>) -> Self::Output {
    Self::Output::new(Add::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<Vec<u8>>>::to(rhs)),
    ))
  }
}

impl<'a, Lhs: Action<Text: ComposeLiteral<&'a [u8]>>> ops::Add<&'a [u8]> for Combinator<Lhs> {
  type Output = Combinator<
    Add<Lhs, Contextual<<Lhs::Text as ComposeLiteral<&'a [u8]>>::Output, Lhs::State, Lhs::Heap>>,
  >;
  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a [u8]) -> Self::Output {
    Self::Output::new(Add::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<&'a [u8]>>::to(rhs)),
    ))
  }
}

impl<'a, const N: usize, Lhs: Action<Text: ComposeLiteral<&'a [u8; N]>>> ops::Add<&'a [u8; N]>
  for Combinator<Lhs>
{
  type Output = Combinator<
    Add<Lhs, Contextual<<Lhs::Text as ComposeLiteral<&'a [u8; N]>>::Output, Lhs::State, Lhs::Heap>>,
  >;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a [u8; N]) -> Self::Output {
    Self::Output::new(Add::new(
      self.action,
      Contextual::new(<Lhs::Text as ComposeLiteral<&'a [u8; N]>>::to(rhs)),
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{bytes, take},
    instant::Instant,
  };
  use std::fmt::Debug;

  fn helper<Text: ?Sized + Digest, Value: PartialEq + Debug>(
    action: impl Action<Text = Text, State = (), Heap = (), Value = Value>,
    input: &Text,
    output: Option<Output<Value>>,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action.exec(Input {
        instant: &Instant::new(input),
        state: &mut (),
        heap: &mut ()
      }),
      output
    )
  }

  #[test]
  fn combinator_add() {
    let rejecter = || take(0).reject(|_| true);
    let accepter_unit = || take(1);
    let accepter_int = || take(1).bind((123,));

    // reject then accept, should return None
    helper(rejecter() + accepter_unit(), "123", None);

    // accept then reject, should return None
    helper(accepter_unit() + rejecter(), "123", None);

    // accept then accept, should return the sum of the digested
    // with the concat value
    helper(
      accepter_unit() + accepter_int(),
      "123",
      Some(Output {
        value: (123,),
        digested: 2,
      }),
    );
    helper(
      accepter_int() + accepter_unit(),
      "123",
      Some(Output {
        value: (123,),
        digested: 2,
      }),
    );
    helper(
      accepter_int() + accepter_int(),
      "123",
      Some(Output {
        value: (123, 123),
        digested: 2,
      }),
    );
  }

  #[test]
  fn combinator_add_char() {
    helper(
      take(1) + '2',
      "12",
      Some(Output {
        digested: 2,
        value: (),
      }),
    );
  }

  #[test]
  fn combinator_add_str() {
    helper(
      take(1) + "23",
      "123",
      Some(Output {
        digested: 3,
        value: (),
      }),
    );
  }

  #[test]
  fn combinator_add_string() {
    helper(
      take(1) + "23".to_string(),
      "123",
      Some(Output {
        digested: 3,
        value: (),
      }),
    );
  }

  #[test]
  fn combinator_add_u8() {
    helper(
      bytes::take(1) + b'2',
      b"123",
      Some(Output {
        digested: 2,
        value: (),
      }),
    );
  }

  #[test]
  fn combinator_add_u8_slice() {
    helper(
      bytes::take(1) + "2".as_bytes(),
      b"123",
      Some(Output {
        digested: 2,
        value: (),
      }),
    );
  }

  #[test]
  fn combinator_add_u8_const_slice() {
    helper(
      bytes::take(1) + b"2",
      b"123",
      Some(Output {
        digested: 2,
        value: (),
      }),
    );
  }

  #[test]
  fn combinator_add_vec_u8() {
    helper(
      bytes::take(1) + vec![b'2'],
      b"123",
      Some(Output {
        digested: 2,
        value: (),
      }),
    );
  }
}
