//! Overload `+` operator for [`Combinator`].
//!
//! `Combinator + Combinator` will create a new combinator
//! to parse with the left-hand side, then parse with the right-hand side.
//! The combinator will return the output with [`Concat`]-ed value
//! and the rest of the input text after the right-hand side is executed,
//! or reject if any of the combinators rejects.
//! # Basics
//! ```
//! # use whitehole::{combinator::eat, C};
//! # fn t(_: C!()) {}
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
//! # use whitehole::{combinator::{next, eat}, C, parser::Builder};
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

use crate::combinator::{Action, Combinator, EatChar, EatStr, EatString, EatUsize, Input, Output};
use std::ops;

/// An [`Action`] created by the `+` operator.
/// See [`ops::add`](crate::combinator::ops::add) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Add<Lhs, Rhs> {
  pub lhs: Lhs,
  pub rhs: Rhs,
}

impl<Lhs, Rhs> Add<Lhs, Rhs> {
  /// Create a new instance with the left-hand side and right-hand side.
  #[inline]
  pub const fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self { lhs, rhs }
  }
}

impl<Lhs: Action<Value: Concat<Rhs::Value>>, Rhs: Action<State = Lhs::State, Heap = Lhs::Heap>>
  Action for Add<Lhs, Rhs>
{
  type Value = <Lhs::Value as Concat<Rhs::Value>>::Output;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.lhs.exec(input).and_then(|output| {
      input
        .reload(output.digested)
        .and_then(|mut input| self.rhs.exec(&mut input))
        .map(|rhs_output| rhs_output.map(|rhs_value| output.value.concat(rhs_value)))
    })
  }
}

impl<Lhs: Action<Value: Concat<Rhs::Value>>, Rhs: Action<State = Lhs::State, Heap = Lhs::Heap>>
  ops::Add<Combinator<Rhs>> for Combinator<Lhs>
{
  type Output = Combinator<Add<Lhs, Rhs>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: Combinator<Rhs>) -> Self::Output {
    Self::Output::new(Add::new(self.action, rhs.action))
  }
}

impl<Lhs: Action<Value: Concat<()>>> ops::Add<char> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatChar<Lhs::State, Lhs::Heap>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: char) -> Self::Output {
    Self::Output::new(Add::new(self.action, EatChar::new(rhs)))
  }
}

impl<Lhs: Action<Value: Concat<()>>> ops::Add<usize> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatUsize<Lhs::State, Lhs::Heap>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: usize) -> Self::Output {
    Self::Output::new(Add::new(self.action, EatUsize::new(rhs)))
  }
}

impl<Lhs: Action<Value: Concat<()>>> ops::Add<String> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatString<Lhs::State, Lhs::Heap>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: String) -> Self::Output {
    Self::Output::new(Add::new(self.action, EatString::new(rhs)))
  }
}

impl<'a, Lhs: Action<Value: Concat<()>>> ops::Add<&'a str> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatStr<'a, Lhs::State, Lhs::Heap>>>;

  /// See [`ops::add`](crate::combinator::ops::add) for more information.
  #[inline]
  fn add(self, rhs: &'a str) -> Self::Output {
    Self::Output::new(Add::new(self.action, EatStr::new(rhs)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::{wrap, Input};

  #[test]
  fn combinator_add() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter_unit = || wrap(|input| input.digest(1));
    let accepter_int = || wrap(|input| input.digest(1).map(|output| output.map(|_| (123,))));

    // reject then accept, should return None
    assert!((rejecter() + accepter_unit())
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // accept then reject, should return None
    assert!((accepter_unit() + rejecter())
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // accept then accept, should return the sum of the digested
    // with the concat value
    assert_eq!(
      (accepter_unit() + accepter_int()).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (123,),
        digested: 2,
      })
    );
    assert_eq!(
      (accepter_int() + accepter_unit()).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (123,),
        digested: 2,
      })
    );
    assert_eq!(
      (accepter_int() + accepter_int()).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
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
        .exec(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(2)
    );
  }

  #[test]
  fn combinator_add_string() {
    let eat1 = || wrap(|input| input.digest(1));

    assert_eq!(
      (eat1() + "23".to_string())
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
  }

  #[test]
  fn combinator_add_str() {
    let eat1 = || wrap(|input| input.digest(1));

    assert_eq!(
      (eat1() + "23")
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
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
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // overflow
    assert_eq!(
      (eat1() + 3)
        .exec(&mut Input::new("1", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // 0
    assert_eq!(
      (eat1() + 0)
        .exec(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(1)
    );
  }
}
