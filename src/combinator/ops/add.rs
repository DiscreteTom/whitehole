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
//! // match "123" then match "456"
//! # t(
//! eat("123") + eat("456")
//! # );
//!
//! // you can use a String, a &str, a char or an usize as a shortcut for `eat`
//! // at the right-hand side of `+`
//! # t(
//! eat("true") + "false".to_string()
//! # );
//! # t(
//! eat("true") + "false"
//! # );
//! # t(
//! eat("true") + ';'
//! # );
//! # t(
//! eat("true") + 1
//! # );
//! ```
//! # Concat Values
//! If your combinators' values are tuples, they can be concatenated,
//! and all unit tuples will be ignored.
//! ```
//! # use whitehole::{combinator::{next, eat}, action::Action, parser::Builder};
//! let integer = || {
//!   (next(|c| c.is_ascii_digit()) * (1..)) // eat one or more digits
//!     .select(|ctx| ctx.content().parse::<usize>().unwrap()) // parse the digits
//!     .tuple() // wrap the parsed digits in a tuple
//! };
//! let dot = eat('.'); // the value is `()`
//!
//! // (usize,) + () + (usize,)
//! // unit tuples will be ignored
//! // so the value will be `(usize, usize)`
//! let entry = integer() + dot + integer();
//!
//! let mut parser = Builder::new().entry(entry).build("123.456");
//! let output = parser.parse().unwrap();
//! assert_eq!(output.value, (123, 456));
//! ```
//! See [`Concat`] for more information.

mod concat;

pub use concat::*;

use crate::{
  combinator::{eat, Action, Combinator, Eat, Input, Output},
  digest::Digest,
};
use std::ops;

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
    Text: ?Sized,
    State,
    Heap,
    Lhs: Action<Text, State, Heap, Value: Concat<Rhs::Value>>,
    Rhs: Action<Text, State, Heap>,
  > Action<Text, State, Heap> for Add<Lhs, Rhs>
where
  for<'a> &'a Text: Digest,
{
  type Value = <Lhs::Value as Concat<Rhs::Value>>::Output;

  #[inline]
  fn exec(&self, mut input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.lhs.exec(input.reborrow()).and_then(|output| {
      self
        .rhs
        .exec(unsafe { input.shift_unchecked(output.digested) })
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

impl<Lhs> ops::Add<usize> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Eat<usize>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: usize) -> Self::Output {
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

impl<'a, Lhs> ops::Add<&'a str> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Eat<&'a str>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a str) -> Self::Output {
    Self::Output::new(Add::new(self.action, eat(rhs).action))
  }
}
impl<'a, Lhs> ops::Add<&'a String> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, Eat<&'a String>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a String) -> Self::Output {
    Self::Output::new(Add::new(self.action, eat(rhs).action))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{wrap, Input},
    instant::Instant,
  };

  #[test]
  fn combinator_add() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter_unit = || wrap(|input| input.digest(1));
    let accepter_int = || wrap(|input| input.digest(1).map(|output| output.map(|_| (123,))));

    // reject then accept, should return None
    assert!((rejecter() + accepter_unit())
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
      .is_none());

    // accept then reject, should return None
    assert!((accepter_unit() + rejecter())
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
      .is_none());

    // accept then accept, should return the sum of the digested
    // with the concat value
    assert_eq!(
      (accepter_unit() + accepter_int()).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (123,),
        digested: 2,
      })
    );
    assert_eq!(
      (accepter_int() + accepter_unit()).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (123,),
        digested: 2,
      })
    );
    assert_eq!(
      (accepter_int() + accepter_int()).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (123, 123),
        digested: 2,
      })
    );
  }

  #[test]
  fn combinator_add_char() {
    let eat1 = || wrap(|input| input.digest(1));

    assert_eq!(
      (eat1() + '2')
        .exec(Input::new(Instant::new("12"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(2)
    );
  }

  #[test]
  fn combinator_add_string() {
    let eat1 = || wrap(|input| input.digest(1));

    assert_eq!(
      (eat1() + "23".to_string())
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
  }

  #[test]
  fn combinator_add_string_ref() {
    let eat1 = || wrap(|input| input.digest(1));

    assert_eq!(
      (eat1() + &"23".to_string())
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
  }

  #[test]
  fn combinator_add_str() {
    let eat1 = || wrap(|input| input.digest(1));

    assert_eq!(
      (eat1() + "23")
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
  }

  #[test]
  fn combinator_add_usize() {
    let eat1 = || wrap(|input| input.digest(1));

    // normal
    assert_eq!(
      (eat1() + 2)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
    // overflow
    assert_eq!(
      (eat1() + 3)
        .exec(Input::new(Instant::new("1"), &mut (), &mut ()))
        .map(|output| output.digested),
      None
    );
    // 0
    assert_eq!(
      (eat1() + 0)
        .exec(Input::new(Instant::new("12"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(1)
    );
  }
}
