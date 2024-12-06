use crate::{combinator::wrap, Combinator};

/// Returns a combinator to match
/// [`Input::next`](crate::parse::Input::next) by the condition.
/// The combinator will reject if not matched.
/// # Examples
/// ```
/// # use whitehole::{combinator::next, Combinator};
/// # fn t(_: Combinator!()) {}
/// // match one ascii digit
/// # t(
/// next(|c| c.is_ascii_digit())
/// # );
/// ```
#[inline]
pub fn next<State, Heap>(condition: impl Fn(char) -> bool) -> Combinator!((), State, Heap) {
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
  use crate::parse::{Input, Parse};

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
