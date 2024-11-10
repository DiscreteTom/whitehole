//! Combinators that match chars by the condition.

use super::eater_unchecked;
use crate::combinator::Combinator;

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
pub fn next<'a, State, Heap>(
  condition: impl Fn(char) -> bool + 'a,
) -> Combinator<'a, (), State, Heap> {
  eater_unchecked(move |input| {
    let next = input.next();
    if !condition(next) {
      return 0;
    }
    next.len_utf8()
  })
}

/// Match chars by the condition greedily.
/// Reject if no char is matched.
///
/// This is usually used with the [`in_str!`](crate::in_str) macro.
/// # Examples
/// ```
/// # use whitehole::{combinator::{Combinator, chars}, in_str};
/// // match all ascii digits greedily
/// let _: Combinator<_> = chars(|ch| ch.is_ascii_digit());
/// // match all JSON whitespaces greedily
/// let _: Combinator<_> = chars(in_str!(" \t\r\n"));
/// ```
pub fn chars<'a, State, Heap>(
  condition: impl Fn(char) -> bool + 'a,
) -> Combinator<'a, (), State, Heap> {
  eater_unchecked(move |input| {
    let mut digested = 0;
    for c in input.rest().chars() {
      if !condition(c) {
        break;
      }
      digested += c.len_utf8();
    }
    digested
  })
}

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
        .map(|output| output.digested),
      Some(1)
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
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(chars(|c| c.is_ascii_alphabetic())
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }
}
