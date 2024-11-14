//! Combinators that match chars by the condition.

use crate::{
  combinator::{Combinator, Input, Output},
  impl_combinator_ops,
};

/// See [`next`].
#[derive(Debug, Clone)]
pub struct Next<F> {
  condition: F,
}

/// Returns a combinator to match the next undigested char by the condition.
/// The combinator will reject if the next char is not matched.
///
/// This is usually used with the [`in_str!`](crate::in_str) macro.
/// # Examples
/// ```
/// # use whitehole::{combinator::{Combinator, next}, in_str};
/// // match one ascii digit
/// next(|c| c.is_ascii_digit());
/// // match a char in a literal str
/// next(in_str!("+-*/"));
/// ```
#[inline]
pub fn next<F: Fn(char) -> bool>(condition: F) -> Next<F> {
  Next { condition }
}

impl<State, Heap, F> Combinator<State, Heap> for Next<F>
where
  F: Fn(char) -> bool,
{
  type Kind = ();

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    let next = input.next();
    if !(self.condition)(next) {
      return None;
    }
    Some(unsafe { input.digest_unchecked(next.len_utf8()) })
  }
}

impl_combinator_ops!(Next<F>, F);

#[derive(Debug, Clone)]
pub struct Chars<F> {
  condition: F,
}

/// Returns a combinator to match chars by the condition greedily.
/// The combinator will reject if no char is matched.
///
/// This is usually used with the [`in_str!`](crate::in_str) macro.
///
/// This has the same behavior as `next(condition) * (1..)` but faster.
/// TODO: make [`next`] faster enough to remove this.
/// # Examples
/// ```
/// # use whitehole::{combinator::{Combinator, chars}, in_str};
/// // match all ascii digits greedily
/// chars(|ch| ch.is_ascii_digit());
/// // match all JSON whitespaces greedily
/// chars(in_str!(" \t\r\n"));
/// ```
#[inline]
pub fn chars<F: Fn(char) -> bool>(condition: F) -> Chars<F> {
  Chars { condition }
}

impl<State, Heap, F> Combinator<State, Heap> for Chars<F>
where
  F: Fn(char) -> bool,
{
  type Kind = ();

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    let mut digested = 0;
    for c in input.rest().chars() {
      if !(self.condition)(c) {
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::Input;

  #[test]
  fn combinator_next() {
    // normal
    assert_eq!(
      next(|c| c.is_ascii_digit())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("23")
    );
    // reject
    assert!(next(|c| c.is_ascii_alphabetic())
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_chars() {
    // normal
    assert_eq!(
      chars(|c| c.is_ascii_digit())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // reject
    assert!(chars(|c| c.is_ascii_alphabetic())
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }
}
