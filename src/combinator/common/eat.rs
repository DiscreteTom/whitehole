//! Basic combinators that just eat some bytes from the input text.

use crate::{
  combinator::{Input, Output, Parse},
  impl_combinator,
};

/// See [`eat`].
#[derive(Debug, Clone, Copy)]
pub struct Eat {
  n: usize,
}

/// Returns a combinator to eat `n` bytes from the head of [`Input::rest`].
/// The combinator will reject if [`Output::rest`] can't be built
/// as a valid UTF-8 string.
///
/// `0` is allowed but be careful with infinite loops.
///
/// # Examples
/// ```
/// use whitehole::combinator::eat;
/// // eat 10 bytes
/// eat(10);
/// ```
#[inline]
pub fn eat(n: usize) -> Eat {
  Eat { n }
}

impl<State, Heap> Parse<State, Heap> for Eat {
  type Kind = ();

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    input.digest(self.n)
  }
}

impl_combinator!(Eat);

/// See [`eat_unchecked`].
#[derive(Debug, Clone, Copy)]
pub struct EatUnchecked {
  n: usize,
}

/// Returns a combinator to eat `n` bytes from the head of [`Input::rest`],
/// without checking `n`.
/// The combinator will never reject.
///
/// `0` is allowed but be careful with infinite loops.
/// # Safety
/// You should ensure that [`Output::rest`] can be built
/// as a valid UTF-8 string.
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`eat`].
/// # Examples
/// ```
/// use whitehole::combinator::eat_unchecked;
/// // eat 10 bytes
/// unsafe { eat_unchecked(10) };
/// ```
#[inline]
pub unsafe fn eat_unchecked(n: usize) -> EatUnchecked {
  EatUnchecked { n }
}

impl<State, Heap> Parse<State, Heap> for EatUnchecked {
  type Kind = ();

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    unsafe { input.digest_unchecked(self.n) }.into()
  }
}

/// See [`eater`].
#[derive(Debug, Clone)]
pub struct Eater<F> {
  f: F,
}

/// Accept a function that eats [`Input::rest`] and returns the number of digested bytes.
/// Reject if the function returns `0` or [`Output::rest`] can't be built
/// as a valid UTF-8 string.
/// # Examples
/// ```
/// use whitehole::combinator::eater;
/// // accept all the rest characters
/// eater(|input| input.rest().len());
/// ```
#[inline]
pub fn eater<State, Heap, F: Fn(&mut Input<&mut State, &mut Heap>) -> usize>(f: F) -> Eater<F> {
  Eater { f }
}

impl<State, Heap, F> Parse<State, Heap> for Eater<F>
where
  F: Fn(&mut Input<&mut State, &mut Heap>) -> usize,
{
  type Kind = ();

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<crate::combinator::Output<'text, ()>> {
    match (self.f)(input) {
      0 => None,
      digested => input.digest(digested),
    }
  }
}

/// See [`eater_unchecked`].
#[derive(Debug, Clone)]
pub struct EaterUnchecked<F> {
  f: F,
}

/// Accept a function that eats [`Input::rest`] and returns the number of digested bytes.
/// Reject if the function returns `0`.
/// # Safety
/// You should ensure that [`Output::rest`] can be built
/// as a valid UTF-8 string.
/// For the checked version, see [`eater`].
/// # Examples
/// ```
/// use whitehole::combinator::eater_unchecked;
/// // accept all the rest characters
/// unsafe { eater_unchecked(|input| input.rest().len()) };
/// ```
#[inline]
pub unsafe fn eater_unchecked<State, Heap, F: Fn(&mut Input<&mut State, &mut Heap>) -> usize>(
  f: F,
) -> EaterUnchecked<F> {
  EaterUnchecked { f }
}

impl<State, Heap, F> Parse<State, Heap> for EaterUnchecked<F>
where
  F: Fn(&mut Input<&mut State, &mut Heap>) -> usize,
{
  type Kind = ();

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<crate::combinator::Output<'text, ()>> {
    match (self.f)(input) {
      0 => None,
      digested => unsafe { input.digest_unchecked(digested) }.into(),
    }
  }
}

impl_combinator!(EaterUnchecked<F>, F);

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn combinator_eat() {
    // normal
    assert_eq!(
      eat(3)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // overflow
    assert_eq!(
      eat(3)
        .parse(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
    // invalid code point
    assert_eq!(
      eat(1)
        .parse(&mut Input::new("好", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
    // 0
    assert_eq!(
      eat(0)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("123")
    );
  }

  #[test]
  fn combinator_eat_unchecked() {
    // normal
    assert_eq!(
      unsafe { eat_unchecked(3) }
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // 0
    assert_eq!(
      unsafe { eat_unchecked(0) }
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("123")
    );
  }

  #[test]
  #[should_panic]
  fn combinator_eat_unchecked_overflow() {
    unsafe { eat_unchecked(3) }.parse(&mut Input::new("12", 0, &mut (), &mut ()).unwrap());
  }

  #[test]
  #[should_panic]
  fn combinator_eat_unchecked_invalid_code_point() {
    unsafe { eat_unchecked(1) }.parse(&mut Input::new("好", 0, &mut (), &mut ()).unwrap());
  }

  #[test]
  fn combinator_eater() {
    // normal
    assert_eq!(
      eater(|input| input.rest().len())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // overflow
    assert_eq!(
      eater(|input| input.rest().len() + 1)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
    // invalid code point
    assert_eq!(
      eater(|_| 1)
        .parse(&mut Input::new("好", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
    // 0
    assert_eq!(
      eater(|_| 0)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
  }

  #[test]
  fn combinator_eater_unchecked() {
    // normal
    assert_eq!(
      unsafe { eater_unchecked(|input| input.rest().len()) }
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // 0
    assert_eq!(
      unsafe { eater_unchecked(|_| 0) }
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
  }

  #[test]
  #[should_panic]
  fn combinator_eater_unchecked_overflow() {
    unsafe { eater_unchecked(|input| input.rest().len() + 1) }
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap());
  }

  #[test]
  #[should_panic]
  fn combinator_eater_unchecked_invalid_code_point() {
    unsafe { eater_unchecked(|_| 1) }.parse(&mut Input::new("好", 0, &mut (), &mut ()).unwrap());
  }
}
