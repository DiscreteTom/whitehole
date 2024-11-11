//! Basic combinators that just eat some bytes from the input text.

use crate::combinator::{Combinator, Input, Output};

/// Eat `n` bytes from the rest of the input text.
/// Reject if [`Output::rest`] can't be built
/// as a valid UTF-8 string.
///
/// `0` is allowed but be careful with infinite loops.
///
/// # Examples
/// ```
/// use whitehole::combinator::{Combinator, eat};
/// // eat 10 bytes
/// let _: Combinator<_> = eat(10);
/// ```
pub fn eat<'a, State, Heap>(n: usize) -> Combinator<'a, (), State, Heap> {
  Combinator::boxed(move |input| input.rest().get(n..).map(|rest| Output { kind: (), rest }))
}

/// Eat `n` bytes from the rest of the input text,
/// without checking `n`.
///
/// `0` is allowed but be careful with infinite loops.
/// # Safety
/// You should ensure that [`Output::rest`] can be built
/// as a valid UTF-8 string.
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`eat`].
/// # Examples
/// ```
/// use whitehole::combinator::{Combinator, eat_unchecked};
/// // eat 10 bytes
/// let _: Combinator<_> = unsafe { eat_unchecked(10) };
/// ```
pub unsafe fn eat_unchecked<'a, State, Heap>(n: usize) -> Combinator<'a, (), State, Heap> {
  Combinator::boxed(move |input| {
    debug_assert!(input.rest().get(n..).is_some());

    Output {
      kind: (),
      rest: input.rest().get_unchecked(n..),
    }
    .into()
  })
}

/// Accept a function that eats [`Input::rest`] and returns the number of digested bytes.
/// Reject if the function returns `0` or [`Output::rest`] can't be built
/// as a valid UTF-8 string.
/// # Examples
/// ```
/// use whitehole::combinator::{Combinator, eater};
/// // accept all the rest characters
/// let _: Combinator<_> = eater(|input| input.rest().len());
/// ```
pub fn eater<'a, State, Heap>(
  f: impl Fn(&mut Input<&mut State, &mut Heap>) -> usize + 'a,
) -> Combinator<'a, (), State, Heap> {
  Combinator::boxed(move |input| match f(input) {
    0 => None,
    digested => (input.rest().get(digested..)).map(|rest| Output { kind: (), rest }),
  })
}

/// Accept a function that eats [`Input::rest`] and returns the number of digested bytes.
/// Reject if the function returns `0`.
/// # Safety
/// You should ensure that [`Output::rest`] can be built
/// as a valid UTF-8 string.
/// For the checked version, see [`eater`].
/// # Examples
/// ```
/// use whitehole::combinator::{Combinator, eater_unchecked};
/// // accept all the rest characters
/// let _: Combinator<_> = unsafe { eater_unchecked(|input| input.rest().len()) };
/// ```
pub unsafe fn eater_unchecked<'a, State, Heap>(
  f: impl Fn(&mut Input<&mut State, &mut Heap>) -> usize + 'a,
) -> Combinator<'a, (), State, Heap> {
  Combinator::boxed(move |input| match f(input) {
    0 => None,
    digested => {
      debug_assert!(input.rest().get(digested..).is_some());
      Output {
        kind: (),
        rest: input.rest().get_unchecked(digested..),
      }
      .into()
    }
  })
}

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
