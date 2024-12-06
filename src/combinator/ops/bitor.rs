//! Overload `|` operator for [`Combinator`].

use crate::combinator::{Combinator, EatChar, EatStr, EatString, EatUsize, Input, Output, Parse};
use std::ops;

/// A [`Parse`] implementor created by `|`.
#[derive(Debug, Clone, Copy)]
pub struct BitOr<Lhs, Rhs> {
  lhs: Lhs,
  rhs: Rhs,
}

impl<Lhs, Rhs> BitOr<Lhs, Rhs> {
  /// Create a new instance with the left-hand side and right-hand side.
  #[inline]
  pub fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self { lhs, rhs }
  }
}

impl<Lhs: Parse, Rhs: Parse<Kind = Lhs::Kind, State = Lhs::State, Heap = Lhs::Heap>> Parse
  for BitOr<Lhs, Rhs>
{
  type Kind = Lhs::Kind;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    self.lhs.parse(input).or_else(|| self.rhs.parse(input))
  }
}

impl<Lhs: Parse, Rhs: Parse<Kind = Lhs::Kind, State = Lhs::State, Heap = Lhs::Heap>>
  ops::BitOr<Combinator<Rhs>> for Combinator<Lhs>
{
  type Output = Combinator<BitOr<Lhs, Rhs>>;

  /// Create a new combinator to try to parse with the left-hand side,
  /// and if it fails, try to parse with the right-hand side.
  /// The combinator will reject if all of the parses reject.
  #[inline]
  fn bitor(self, rhs: Combinator<Rhs>) -> Self::Output {
    Self::Output::new(BitOr::new(self.parser, rhs.parser))
  }
}

impl<Lhs: Parse> ops::BitOr<char> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, EatChar<Lhs::State, Lhs::Heap>>>;

  /// Similar to `self | eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn bitor(self, rhs: char) -> Self::Output {
    Self::Output::new(BitOr::new(self.parser, EatChar::new(rhs)))
  }
}

impl<Lhs: Parse> ops::BitOr<usize> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, EatUsize<Lhs::State, Lhs::Heap>>>;

  /// Similar to `self | eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn bitor(self, rhs: usize) -> Self::Output {
    Self::Output::new(BitOr::new(self.parser, EatUsize::new(rhs)))
  }
}

impl<Lhs: Parse> ops::BitOr<String> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, EatString<Lhs::State, Lhs::Heap>>>;

  /// Similar to `self | eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn bitor(self, rhs: String) -> Self::Output {
    Self::Output::new(BitOr::new(self.parser, EatString::new(rhs)))
  }
}

impl<'a, Lhs: Parse> ops::BitOr<&'a str> for Combinator<Lhs> {
  type Output = Combinator<BitOr<Lhs, EatStr<'a, Lhs::State, Lhs::Heap>>>;

  /// Similar to `self | eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn bitor(self, rhs: &'a str) -> Self::Output {
    Self::Output::new(BitOr::new(self.parser, EatStr::new(rhs)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::{wrap, Input, Output};

  #[test]
  fn combinator_bit_or() {
    let mut state = 0;

    let rejecter = || {
      wrap(|input| {
        *input.state += 1;
        None
      })
    };
    let accepter = || {
      wrap(|input| {
        *input.state += 1;
        Some(Output {
          kind: (),
          rest: &input.rest()[1..],
        })
      })
    };

    // reject then accept, both should increment the state
    assert_eq!(
      (rejecter() | accepter()).parse(&mut Input::new("123", 0, &mut state, &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "23",
      })
    );
    assert_eq!(state, 2);

    state = 0;

    // accept then reject, only the first should increment the state
    assert_eq!(
      (accepter() | rejecter()).parse(&mut Input::new("123", 0, &mut state, &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "23",
      })
    );
    assert_eq!(state, 1);
  }

  #[test]
  fn combinator_bit_or_char() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | '1')
        .parse(&mut Input::new("1", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
  }

  #[test]
  fn combinator_bit_or_usize() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | 1)
        .parse(&mut Input::new("1", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
  }

  #[test]
  fn combinator_bit_or_str() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | "1")
        .parse(&mut Input::new("1", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
  }

  #[test]
  fn combinator_bit_or_string() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    assert_eq!(
      (rejecter() | "1".to_string())
        .parse(&mut Input::new("1", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
  }
}
