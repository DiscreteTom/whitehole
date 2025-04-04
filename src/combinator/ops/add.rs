//! Overload `+` operator for [`Combinator`].
//!
//! `Combinator + Combinator` will create a new combinator
//! to parse with the left-hand side, then parse with the right-hand side.
//! The combinator will return the output with [`Concat`]-ed value
//! and the rest of the input text after the right-hand side is executed,
//! or reject if any of the combinators rejects.
//! # Basics
//! ```
//! # use whitehole::{combinator::{eat, bytes, Combinator}, action::Action};
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
//! bytes::eat(b"true") + b'a'
//! # );
//! # tb(
//! bytes::eat(b"true") + b"false"
//! # );
//! # tb(
//! bytes::eat(b"true") + b"false".to_vec()
//! # );
//! ```
//! # Concat Values
//! If your combinators' values are tuples, they can be concatenated,
//! and all unit tuples will be ignored.
//! ```
//! # use whitehole::{combinator::{next, eat}, action::Action, parser::Parser};
//! let integer = || {
//!   (next(|c| c.is_ascii_digit()) * (1..)) // eat one or more digits
//!     .select(|accepted| accepted.content().parse::<usize>().unwrap()) // parse the digits
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

impl<Lhs, Rhs> ops::Add<Combinator<Rhs>> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Rhs>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: Combinator<Rhs>) -> Self::Output {
    Self::Output::new(Add::new(self.action, rhs.action))
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

impl<Lhs: Action<Text = str>> ops::Add<char> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Contextual<Eat<char>, Lhs::State, Lhs::Heap>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: char) -> Self::Output {
    Self::Output::new(Add::new(self.action, Contextual::new(Eat::new(rhs))))
  }
}

impl<Lhs: Action<Text = str>> ops::Add<String> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Contextual<Eat<String>, Lhs::State, Lhs::Heap>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: String) -> Self::Output {
    Self::Output::new(Add::new(self.action, Contextual::new(Eat::new(rhs))))
  }
}

impl<'a, Lhs: Action<Text = str>> ops::Add<&'a str> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Contextual<Eat<&'a str>, Lhs::State, Lhs::Heap>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a str) -> Self::Output {
    Self::Output::new(Add::new(self.action, Contextual::new(Eat::new(rhs))))
  }
}

impl<Lhs: Action<Text = [u8]>> ops::Add<u8> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Contextual<bytes::Eat<u8>, Lhs::State, Lhs::Heap>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: u8) -> Self::Output {
    Self::Output::new(Add::new(self.action, Contextual::new(bytes::Eat::new(rhs))))
  }
}

impl<Lhs: Action<Text = [u8]>> ops::Add<Vec<u8>> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Contextual<bytes::Eat<Vec<u8>>, Lhs::State, Lhs::Heap>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: Vec<u8>) -> Self::Output {
    Self::Output::new(Add::new(self.action, Contextual::new(bytes::Eat::new(rhs))))
  }
}

impl<'a, Lhs: Action<Text = [u8]>> ops::Add<&'a [u8]> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Contextual<bytes::Eat<&'a [u8]>, Lhs::State, Lhs::Heap>>>;
  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a [u8]) -> Self::Output {
    Self::Output::new(Add::new(self.action, Contextual::new(bytes::Eat::new(rhs))))
  }
}

impl<'a, const N: usize, Lhs: Action<Text = [u8]>> ops::Add<&'a [u8; N]> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Contextual<bytes::Eat<&'a [u8; N]>, Lhs::State, Lhs::Heap>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a [u8; N]) -> Self::Output {
    Self::Output::new(Add::new(self.action, Contextual::new(bytes::Eat::new(rhs))))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{bytes, take},
    contextual,
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

  fn _with_contextual() {
    contextual!(i32, i32);

    fn validate(_: impl Action<State = i32, Heap = i32>) {}

    validate(take(1) + 'a'); // char
    validate(take(1) + "a"); // &str
    validate(take(1) + "a".to_string()); // String
    validate(bytes::take(1) + b'a'); // u8
    validate(bytes::take(1) + b"a"); // &[u8; N]
    validate(bytes::take(1) + b"a".as_bytes()); // &[u8]
    validate(bytes::take(1) + b"a".to_vec()); // Vec<u8>
  }
}
