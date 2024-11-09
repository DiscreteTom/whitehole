use super::eater_unchecked;
use crate::combinator::Combinator;

/// Match chars by the condition greedily.
/// Reject if no char is matched.
///
/// # Examples
/// ```
/// # use whitehole::combinator::{Combinator, chars};
/// // match all ascii digits greedily
/// let _: Combinator<_> = chars(|ch| ch.is_ascii_digit());
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
  fn combinator_chars() {
    // normal
    assert_eq!(
      chars(|_| true)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(chars(|_| false)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }
}
