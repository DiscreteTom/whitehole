//! Overload `+` operator for [`Combinator`].

mod concat;

pub use concat::*;

use crate::combinator::{Action, Combinator, EatChar, EatStr, EatString, EatUsize, Input, Output};
use std::ops;

/// An [`Action`] created by `+`.
#[derive(Debug, Clone, Copy)]
pub struct Add<Lhs, Rhs> {
  lhs: Lhs,
  rhs: Rhs,
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
  ) -> Option<Output<'text, Self::Value>> {
    self.lhs.exec(input).and_then(|output| {
      input
        .reload(output.rest)
        .and_then(|mut input| self.rhs.exec(&mut input))
        .map(|rhs_output| Output {
          value: output.value.concat(rhs_output.value),
          rest: rhs_output.rest,
        })
    })
  }
}

impl<Lhs: Action<Value: Concat<Rhs::Value>>, Rhs: Action<State = Lhs::State, Heap = Lhs::Heap>>
  ops::Add<Combinator<Rhs>> for Combinator<Lhs>
{
  type Output = Combinator<Add<Lhs, Rhs>>;

  /// Create a new combinator to parse with the left-hand side, then parse with the right-hand side.
  /// The combinator will return the output with [`Concat`]-ed value and the sum of the digested,
  /// or reject if any of the parses rejects.
  #[inline]
  fn add(self, rhs: Combinator<Rhs>) -> Self::Output {
    Self::Output::new(Add::new(self.action, rhs.action))
  }
}

impl<Lhs: Action<Value: Concat<()>>> ops::Add<char> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatChar<Lhs::State, Lhs::Heap>>>;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn add(self, rhs: char) -> Self::Output {
    Self::Output::new(Add::new(self.action, EatChar::new(rhs)))
  }
}

impl<Lhs: Action<Value: Concat<()>>> ops::Add<usize> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatUsize<Lhs::State, Lhs::Heap>>>;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn add(self, rhs: usize) -> Self::Output {
    Self::Output::new(Add::new(self.action, EatUsize::new(rhs)))
  }
}

impl<Lhs: Action<Value: Concat<()>>> ops::Add<String> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatString<Lhs::State, Lhs::Heap>>>;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn add(self, rhs: String) -> Self::Output {
    Self::Output::new(Add::new(self.action, EatString::new(rhs)))
  }
}

impl<'a, Lhs: Action<Value: Concat<()>>> ops::Add<&'a str> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatStr<'a, Lhs::State, Lhs::Heap>>>;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
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
        rest: "3",
      })
    );
    assert_eq!(
      (accepter_int() + accepter_unit()).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (123,),
        rest: "3",
      })
    );
    assert_eq!(
      (accepter_int() + accepter_int()).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (123, 123),
        rest: "3",
      })
    );
  }

  #[test]
  fn combinator_add_char() {
    let eat1 = || wrap(|input| input.digest(1));

    assert_eq!(
      (eat1() + '2')
        .exec(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
  }

  #[test]
  fn combinator_add_string() {
    let eat1 = || wrap(|input| input.digest(1));

    assert_eq!(
      (eat1() + "23".to_string())
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
  }

  #[test]
  fn combinator_add_str() {
    let eat1 = || wrap(|input| input.digest(1));

    assert_eq!(
      (eat1() + "23")
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
  }

  #[test]
  fn combinator_add_usize() {
    let eat1 = || wrap(|input| input.digest(1));

    // normal
    assert_eq!(
      (eat1() + 2)
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // overflow
    assert_eq!(
      (eat1() + 3)
        .exec(&mut Input::new("1", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
    // 0
    assert_eq!(
      (eat1() + 0)
        .exec(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("2")
    );
  }
}
