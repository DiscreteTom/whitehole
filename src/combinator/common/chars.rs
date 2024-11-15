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
) -> Combinator<impl Parse<Kind = (), State = State, Heap = Heap>> {
  wrap(move |input| {
    let next = input.next();
    if !condition(next) {
      return None;
    }
    Some(unsafe { input.digest_unchecked(next.len_utf8()) })
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
}
