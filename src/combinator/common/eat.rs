use crate::combinator::{Combinator, Input, Output};

/// Eat `n` bytes from the rest of the input text.
/// Reject if there are less than `n` bytes left.
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
  Combinator::boxed(move |input| {
    (n <= input.rest().len()).then_some(Output {
      kind: (),
      digested: n,
    })
  })
}

/// Eat `n` bytes from the rest of the input text,
/// without checking the length of the rest
/// (so this combinator will never reject).
///
/// `0` is allowed but be careful with infinite loops.
/// # Caveats
/// You should ensure that `n` is no greater than the length of [`Input::rest`].
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`eat`].
/// # Examples
/// ```
/// use whitehole::combinator::{Combinator, eat_unchecked};
/// // eat 10 bytes
/// let _: Combinator<_> = eat_unchecked(10);
/// ```
pub fn eat_unchecked<'a, State, Heap>(n: usize) -> Combinator<'a, (), State, Heap> {
  Combinator::boxed(move |input| {
    debug_assert!(n <= input.rest().len());

    Output {
      kind: (),
      digested: n,
    }
    .into()
  })
}

/// Accept a function that eats the rest of the input text and returns the number of digested bytes.
/// Reject if the function returns `0` or the return value is greater than the length of the rest.
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
    digested => (digested <= input.rest().len()).then_some(Output { kind: (), digested }),
  })
}

/// Accept a function that eats the rest of the input text and returns the number of digested bytes.
/// Reject if the function returns `0`.
/// # Caveats
/// You should ensure that the return value is no greater than the length of [`Input::rest`].
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`eater`].
/// # Examples
/// ```
/// use whitehole::combinator::{Combinator, eater_unchecked};
/// // accept all the rest characters
/// let _: Combinator<_> = eater_unchecked(|input| input.rest().len());
/// ```
pub fn eater_unchecked<'a, State, Heap>(
  f: impl Fn(&mut Input<&mut State, &mut Heap>) -> usize + 'a,
) -> Combinator<'a, (), State, Heap> {
  Combinator::boxed(move |input| match f(input) {
    0 => None,
    digested => {
      debug_assert!(digested <= input.rest().len());
      Output { kind: (), digested }.into()
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
        .map(|output| output.digested),
      Some(3)
    );
    // overflow
    assert_eq!(
      eat(3)
        .parse(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // 0
    assert_eq!(
      eat(0)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(0)
    );
  }

  #[test]
  fn combinator_eat_unchecked() {
    // normal
    assert_eq!(
      eat_unchecked(3)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // 0
    assert_eq!(
      eat_unchecked(0)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(0)
    );
  }

  #[test]
  #[should_panic]
  fn combinator_eat_unchecked_overflow() {
    eat_unchecked(3).parse(&mut Input::new("12", 0, &mut (), &mut ()).unwrap());
  }

  #[test]
  fn combinator_eater() {
    // normal
    assert_eq!(
      eater(|input| input.rest().len())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // overflow
    assert_eq!(
      eater(|input| input.rest().len() + 1)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // 0
    assert_eq!(
      eater(|_| 0)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
  }

  #[test]
  fn combinator_eater_unchecked() {
    // normal
    assert_eq!(
      eater_unchecked(|input| input.rest().len())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // 0
    assert_eq!(
      eater_unchecked(|_| 0)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
  }

  #[test]
  #[should_panic]
  fn combinator_eater_unchecked_overflow() {
    eater_unchecked(|input| input.rest().len() + 1)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap());
  }
}
