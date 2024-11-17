//! Overload [`BitOr`] operator for combinator.

use super::{EatChar, EatStr, EatString, EatUsize};
use crate::combinator::{Combinator, Input, Output, Parse};
use std::ops;

/// A [`Parse`] implementor created by `|`.
#[derive(Debug, Clone, Copy)]
pub struct BitOr<Lhs, Rhs> {
  lhs: Lhs,
  rhs: Rhs,
}

impl<Lhs, Rhs> BitOr<Lhs, Rhs> {
  #[inline]
  pub fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self { lhs, rhs }
  }
}

impl<Kind, State, Heap, Lhs: Parse<Kind, State, Heap>, Rhs: Parse<Kind, State, Heap>>
  Parse<Kind, State, Heap> for BitOr<Lhs, Rhs>
{
  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Kind>> {
    self.lhs.parse(input).or_else(|| self.rhs.parse(input))
  }
}

impl<Kind, State, Heap, Lhs: Parse<Kind, State, Heap>, Rhs: Parse<Kind, State, Heap>>
  ops::BitOr<Combinator<Kind, State, Heap, Rhs>> for Combinator<Kind, State, Heap, Lhs>
{
  type Output = Combinator<Kind, State, Heap, BitOr<Lhs, Rhs>>;

  /// Try to parse with the left-hand side, if it fails, try the right-hand side.
  #[inline]
  fn bitor(self, rhs: Combinator<Kind, State, Heap, Rhs>) -> Self::Output {
    Self::Output::new(BitOr::new(self.parser, rhs.parser))
  }
}

impl<Kind, State, Heap, Lhs: Parse<Kind, State, Heap>> ops::BitOr<char>
  for Combinator<Kind, State, Heap, Lhs>
{
  type Output = Combinator<Kind, State, Heap, BitOr<Lhs, EatChar<State, Heap>>>;

  /// Similar to `self | eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn bitor(self, rhs: char) -> Self::Output {
    Self::Output::new(BitOr::new(self.parser, EatChar::new(rhs)))
  }
}

impl<Kind, State, Heap, Lhs: Parse<Kind, State, Heap>> ops::BitOr<usize>
  for Combinator<Kind, State, Heap, Lhs>
{
  type Output = Combinator<Kind, State, Heap, BitOr<Lhs, EatUsize<State, Heap>>>;

  /// Similar to `self | eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn bitor(self, rhs: usize) -> Self::Output {
    Self::Output::new(BitOr::new(self.parser, EatUsize::new(rhs)))
  }
}

impl<Kind, State, Heap, Lhs: Parse<Kind, State, Heap>> ops::BitOr<String>
  for Combinator<Kind, State, Heap, Lhs>
{
  type Output = Combinator<Kind, State, Heap, BitOr<Lhs, EatString<State, Heap>>>;

  /// Similar to `self | eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn bitor(self, rhs: String) -> Self::Output {
    Self::Output::new(BitOr::new(self.parser, EatString::new(rhs)))
  }
}

impl<'a, Kind, State, Heap, Lhs: Parse<Kind, State, Heap>> ops::BitOr<&'a str>
  for Combinator<Kind, State, Heap, Lhs>
{
  type Output = Combinator<Kind, State, Heap, BitOr<Lhs, EatStr<'a, State, Heap>>>;

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
