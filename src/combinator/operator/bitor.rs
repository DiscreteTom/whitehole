//! Overload [`BitOr`] operator for combinator.

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

impl<Lhs: Parse, Rhs: Parse<Kind = Lhs::Kind, State = Lhs::State, Heap = Lhs::Heap>> ops::BitOr<Rhs>
  for Combinator<Lhs>
{
  type Output = Combinator<BitOr<Lhs, Rhs>>;

  /// Try to parse with the left-hand side, if it fails, try the right-hand side.
  #[inline]
  fn bitor(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(BitOr::new(self.parser, rhs))
  }
}

// TODO: impl Combinator and Parse for char, String and &str?
// impl<'a, State: 'a, Heap: 'a, T: Exact + 'a> ops::BitOr<T> for Combinator< State, Heap> {
//   type Output = Combinator<'a, (), State, Heap>;

//   /// Shortcut for `self | exact(rhs)`. See [`exact`].
//   #[inline]
//   fn bitor(self, rhs: T) -> Self::Output {
//     self | exact(rhs)
//   }
// }

// impl<'a, State: 'a, Heap: 'a> ops::BitOr<usize> for Combinator<'a, (), State, Heap> {
//   type Output = Combinator<'a, (), State, Heap>;

//   /// Shortcut for `self | eat(rhs)`. See [`eat`].
//   #[inline]
//   fn bitor(self, rhs: usize) -> Self::Output {
//     self | eat(rhs)
//   }
// }

// TODO: restore tests
// #[cfg(test)]
// mod tests {
//   use super::*;
//   use crate::combinator::{Input, Output};

//   #[test]
//   fn combinator_bit_or() {
//     let mut state = 0;

//     let rejecter = || {
//       Combinator::boxed(|input| {
//         *input.state += 1;
//         None
//       })
//     };
//     let accepter = || {
//       Combinator::boxed(|input| {
//         *input.state += 1;
//         Some(Output {
//           kind: (),
//           rest: &input.rest()[1..],
//         })
//       })
//     };

//     // reject then accept, both should increment the state
//     assert_eq!(
//       (rejecter() | accepter()).parse(&mut Input::new("123", 0, &mut state, &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "23",
//       })
//     );
//     assert_eq!(state, 2);

//     state = 0;

//     // accept then reject, only the first should increment the state
//     assert_eq!(
//       (accepter() | rejecter()).parse(&mut Input::new("123", 0, &mut state, &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "23",
//       })
//     );
//     assert_eq!(state, 1);
//   }

//   fn _combinator_bit_or_exact_prefix() {
//     let _: Combinator<_> = Combinator::boxed(|_| None) | "123"; // with &str
//     let _: Combinator<_> = Combinator::boxed(|_| None) | "123".to_string(); // with String
//     let _: Combinator<_> = Combinator::boxed(|_| None) | '1'; // with char
//   }

//   fn _combinator_bit_or_usize() {
//     let _: Combinator<_> = Combinator::boxed(|_| None) | 1;
//   }
// }
