use crate::{combinator::wrap_unchecked, C};

/// Returns a combinator to match
/// [`Input::next`](crate::action::Input::next) by the condition.
/// The combinator will reject if not matched.
/// # Examples
/// ```
/// # use whitehole::{combinator::next, C};
/// # fn t(_: C!()) {}
/// // match one ascii digit
/// # t(
/// next(|c| c.is_ascii_digit())
/// # );
/// // match one or more ascii digit
/// # t(
/// next(|c| c.is_ascii_digit()) * (1..)
/// # );
/// ```
#[inline]
pub const fn next<State, Heap>(condition: impl Fn(char) -> bool) -> C!((), State, Heap) {
  unsafe {
    wrap_unchecked(move |input| {
      let next = input.next();
      if !condition(next) {
        return None;
      }
      Some(input.digest_unchecked(next.len_utf8()))
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    action::{Action, Input},
    instant::Instant,
  };

  #[test]
  fn combinator_next() {
    // normal
    assert_eq!(
      next(|c| c.is_ascii_digit())
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(1)
    );
    // reject
    assert!(next(|c| c.is_ascii_alphabetic())
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn one_or_more_next() {
    // normal
    assert_eq!(
      (next(|c| c.is_ascii_digit()) * (1..))
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(next(|c| c.is_ascii_digit())
      .exec(Input::new(Instant::new("abc"), &mut (), &mut ()).unwrap())
      .is_none());
  }
}
