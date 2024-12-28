use super::wrap_unchecked;
use crate::{action::Input, C};

/// Returns a combinator by the provided function that
/// eats [`Input::rest`] and returns the number of digested bytes (not chars).
/// The combinator will reject if the function returns `0`
/// or [`Output::digested`](crate::action::Output::digested) is invalid.
/// # Examples
/// ```
/// # use whitehole::{combinator::eater, C};
/// # fn t(_: C!()) {}
/// // accept all the rest characters
/// # t(
/// eater(|input| input.rest().len())
/// # );
/// ```
#[inline]
pub fn eater<State, Heap>(
  f: impl Fn(Input<&mut State, &mut Heap>) -> usize,
) -> C!((), State, Heap) {
  unsafe {
    wrap_unchecked(move |mut input| match f(input.reborrow()) {
      0 => None,
      digested => input.digest(digested),
    })
  }
}

/// Returns a combinator by the provided function that
/// eats [`Input::rest`] and returns the number of digested bytes (not chars).
/// The combinator will reject if the function returns `0`.
/// # Safety
/// You should ensure that the [`Output::digested`](crate::action::Output::digested) is valid.
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`eater`].
/// # Examples
/// ```
/// # use whitehole::{combinator::eater_unchecked, C};
/// # fn t(_: C!()) {}
/// // accept all the rest characters
/// # t(
/// unsafe { eater_unchecked(|input| input.rest().len()) }
/// # );
/// ```
#[inline]
pub unsafe fn eater_unchecked<State, Heap>(
  f: impl Fn(Input<&mut State, &mut Heap>) -> usize,
) -> C!((), State, Heap) {
  wrap_unchecked(move |mut input| match f(input.reborrow()) {
    0 => None,
    digested => input.digest_unchecked(digested).into(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::action::Action;

  #[test]
  fn combinator_eater() {
    // normal
    assert_eq!(
      eater(|input| input.rest().len())
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // overflow
    assert_eq!(
      eater(|input| input.rest().len() + 1)
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // invalid code point
    assert_eq!(
      eater(|_| 1)
        .exec(Input::new("好", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // 0
    assert_eq!(
      eater(|_| 0)
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
  }

  #[test]
  fn combinator_eater_unchecked() {
    // normal
    assert_eq!(
      unsafe { eater_unchecked(|input| input.rest().len()) }
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // 0
    assert_eq!(
      unsafe { eater_unchecked(|_| 0) }
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
  }

  #[test]
  #[should_panic]
  fn combinator_eater_unchecked_overflow() {
    unsafe { eater_unchecked(|input| input.rest().len() + 1) }
      .exec(Input::new("123", 0, &mut (), &mut ()).unwrap());
  }

  #[test]
  #[should_panic]
  fn combinator_eater_unchecked_invalid_code_point() {
    unsafe { eater_unchecked(|_| 1) }.exec(Input::new("好", 0, &mut (), &mut ()).unwrap());
  }
}
