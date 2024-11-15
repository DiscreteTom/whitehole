//! Combinators that match chars by the condition.

use crate::combinator::{wrap, Combinator, Parse};

/// Returns a combinator to match the next undigested char by the condition.
/// The combinator will reject if the next char is not matched.
///
/// This is usually used with the [`in_str!`](crate::in_str) macro.
/// # Examples
/// ```
/// use whitehole::{combinator::next, in_str};
/// // match one ascii digit
/// next(|c| c.is_ascii_digit());
/// // match a char in a literal str
/// next(in_str!("+-*/"));
/// ```
#[inline]
pub fn next<State, Heap, F: Fn(char) -> bool>(
  condition: F,
) -> Combinator<impl Parse<State, Heap, Kind = ()>> {
  wrap(move |input| {
    let next = input.next();
    if !condition(next) {
      return None;
    }
    Some(unsafe { input.digest_unchecked(next.len_utf8()) })
  })
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
/// use whitehole::{combinator::chars, in_str};
/// // match all ascii digits greedily
/// chars(|ch| ch.is_ascii_digit());
/// // match all JSON whitespaces greedily
/// chars(in_str!(" \t\r\n"));
/// ```
#[inline]
pub fn chars<State, Heap, F: Fn(char) -> bool>(
  condition: F,
) -> Combinator<impl Parse<State, Heap, Kind = ()>> {
  wrap(move |input| {
    let mut digested = 0;
    for c in input.rest().chars() {
      if !condition(c) {
        break;
      }
      digested += c.len_utf8();
    }
    if digested == 0 {
      return None;
    }
    Some(unsafe { input.digest_unchecked(digested) })
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
