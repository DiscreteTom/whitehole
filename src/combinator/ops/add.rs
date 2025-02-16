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
        .exec(
          &unsafe { instant.to_digested_unchecked(output.digested) },
          ctx,
        )
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
  use crate::{combinator::take, instant::Instant};
  use std::fmt::Debug;

  fn helper<Text: ?Sized + Digest, Value: PartialEq + Debug>(
    action: impl Action<Text, Value = Value>,
    input: &Text,
    output: Option<Output<Value>>,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action.exec(
        &Instant::new(input),
        Context {
          state: &mut (),
          heap: &mut ()
        }
      ),
      output
    )
  }

  #[test]
  fn combinator_add() {
    let rejecter = || take(0).reject(|_, _| true);
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
      take(1) + b'2',
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
      take(1) + "2".as_bytes(),
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
      take(1) + b"2",
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
      take(1) + vec![b'2'],
      b"123",
      Some(Output {
        digested: 2,
        value: (),
      }),
    );
  }
}
