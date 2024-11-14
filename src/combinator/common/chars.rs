//! Combinators that match chars by the condition.

use std::ops;

use super::eater_unchecked;
use crate::{
  combinator::{
    operator::mul::{Mul, Repeat},
    Combinator, Input, Output,
  },
  impl_combinator_ops,
};

pub struct Next<F> {
  f: F,
}

/// Match the next undigested char by the condition.
/// Reject if the char is not matched.
///
/// This is usually used with the [`in_str!`](crate::in_str) macro.
/// # Examples
/// ```
/// # use whitehole::{combinator::{Combinator, next}, in_str};
/// // match one ascii digit
/// let _: Combinator<_> = next(|ch| ch.is_ascii_digit());
/// // match a char in a literal str
/// let _: Combinator<_> = next(in_str!("+-*/"));
/// ```
pub fn next<F: Fn(char) -> bool>(condition: F) -> Next<F> {
  Next { f: condition }
}

impl<State, Heap, F> Combinator<State, Heap> for Next<F>
where
  F: Fn(char) -> bool,
{
  type Kind = ();

  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    let next = input.next();
    if !(self.f)(next) {
      return None;
    }
    Some(unsafe { input.digest_unchecked(next.len_utf8()) })
  }
}

impl_combinator_ops!(Next<F>, F);

pub struct Chars<F> {
  f: F,
}

/// Match chars by the condition greedily.
/// Reject if no char is matched.
///
/// This is usually used with the [`in_str!`](crate::in_str) macro.
///
/// Currently this equals to `next(condition) * (1..)` but faster.
/// TODO: make next faster enough to remove this.
/// # Examples
/// ```
/// # use whitehole::{combinator::{Combinator, chars}, in_str};
/// // match all ascii digits greedily
/// let _: Combinator<_> = chars(|ch| ch.is_ascii_digit());
/// // match all JSON whitespaces greedily
/// let _: Combinator<_> = chars(in_str!(" \t\r\n"));
/// ```
pub fn chars<F: Fn(char) -> bool>(condition: F) -> Chars<F> {
  Chars { f: condition }
}

impl<State, Heap, F> Combinator<State, Heap> for Chars<F>
where
  F: Fn(char) -> bool,
{
  type Kind = ();

  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    let mut digested = 0;
    for c in input.rest().chars() {
      if !(self.f)(c) {
        break;
      }
      digested += c.len_utf8();
    }
    if digested == 0 {
      return None;
    }
    Some(unsafe { input.digest_unchecked(digested) })
  }
}

impl_combinator_ops!(Chars<F>, F);

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use crate::combinator::Input;

//   #[test]
//   fn combinator_next() {
//     // normal
//     assert_eq!(
//       next(|c| c.is_ascii_digit())
//         .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//         .map(|output| output.rest),
//       Some("23")
//     );
//     // reject
//     assert!(next(|c| c.is_ascii_alphabetic())
//       .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//       .is_none());
//   }

//   #[test]
//   fn combinator_chars() {
//     // normal
//     assert_eq!(
//       chars(|c| c.is_ascii_digit())
//         .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//         .map(|output| output.rest),
//       Some("")
//     );
//     // reject
//     assert!(chars(|c| c.is_ascii_alphabetic())
//       .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//       .is_none());
//   }
// }
