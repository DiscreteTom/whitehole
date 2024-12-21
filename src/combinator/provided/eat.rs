use crate::{
  action::Action,
  combinator::{wrap, Input, Output},
  Combinator,
};
use std::marker::PhantomData;

/// A util trait to make [`eat`] generic over different types.
///
/// Built-in implementations are provided for [`String`], `&str`, [`char`] and [`usize`].
///
/// See [`eat`] for more details.
pub trait Eat {
  /// Convert the implementor to a parser implementor.
  fn into_parser<State, Heap>(self) -> impl Action<Value = (), State = State, Heap = Heap>;
}

macro_rules! impl_eat {
  ($name:ident, $inner:ty, ($($derive:ident),*)) => {
    /// An [`Eat`] implementor.
    /// For most cases you don't need to use this directly.
    ///
    /// See [`eat`] for more details.
    #[derive(Debug, Clone, $($derive),*)]
    pub struct $name<State = (), Heap = ()> {
      inner: $inner,
      _phantom: PhantomData<(State, Heap)>,
    }

    impl<State, Heap> $name<State, Heap> {
      /// Create a new instance with the inner value.
      #[inline]
      pub const fn new(inner: $inner) -> Self {
        Self {
          inner,
          _phantom: PhantomData,
        }
      }
    }

    impl Eat for $inner {
      #[inline]
      fn into_parser<State, Heap>(self) -> impl Action<Value = (), State=State, Heap=Heap> {
        $name::new(self)
      }
    }
  };
}

impl_eat!(EatChar, char, (Copy));
impl_eat!(EatString, String, ());
impl_eat!(EatUsize, usize, (Copy));

impl<State, Heap> Action for EatChar<State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    input
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { input.digest_unchecked(self.inner.len_utf8()) })
  }
}

impl<State, Heap> Action for EatString<State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    input
      .rest()
      .starts_with(&self.inner)
      .then(|| unsafe { input.digest_unchecked(self.inner.len()) })
  }
}

impl<State, Heap> Action for EatUsize<State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    input.digest(self.inner)
  }
}

/// An [`Eat`] implementor.
/// For most cases you don't need to use this directly.
///
/// See [`eat`] for more details.
#[derive(Debug, Clone, Copy)]
pub struct EatStr<'a, State = (), Heap = ()> {
  s: &'a str,
  _phantom: PhantomData<(State, Heap)>,
}

impl<'a, State, Heap> EatStr<'a, State, Heap> {
  /// Create a new instance with the inner value.
  #[inline]
  pub const fn new(s: &'a str) -> Self {
    Self {
      s,
      _phantom: PhantomData,
    }
  }
}

impl<'a> Eat for &'a str {
  #[inline]
  fn into_parser<State, Heap>(self) -> impl Action<Value = (), State = State, Heap = Heap> {
    EatStr::new(self)
  }
}

impl<State, Heap> Action for EatStr<'_, State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    input
      .rest()
      .starts_with(self.s)
      .then(|| unsafe { input.digest_unchecked(self.s.len()) })
  }
}

/// Returns a combinator to eat from the head of [`Input::rest`] by the provided pattern.
/// The combinator will reject if [`Output::rest`] can't be built
/// as a valid UTF-8 string.
///
/// `0` and `""` (empty string) are allowed but be careful with infinite loops.
///
/// # Examples
/// ```
/// # use whitehole::{combinator::eat, Combinator};
/// # fn t(_: Combinator!()) {}
/// # t(
/// eat('a') // eat by char
/// # );
/// # t(
/// eat("true") // eat by &str
/// # );
/// # t(
/// eat("true".to_string()) // eat by String
/// # );
/// # t(
/// eat(10) // eat by byte length
/// # );
/// ```
#[inline]
pub fn eat<State, Heap>(pattern: impl Eat) -> Combinator!((), State, Heap) {
  let parser = pattern.into_parser();
  wrap(move |input| parser.exec(input))
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
/// # use whitehole::{combinator::eat_unchecked, Combinator};
/// # fn t(_: Combinator!()) {}
/// // eat 10 bytes
/// # t(
/// unsafe { eat_unchecked(10) }
/// # );
/// ```
#[inline]
pub unsafe fn eat_unchecked<State, Heap>(n: usize) -> Combinator!((), State, Heap) {
  wrap(move |input| input.digest_unchecked(n).into())
}

/// Returns a combinator by the provided function that
/// eats [`Input::rest`] and returns the number of digested bytes.
/// The combinator will reject if the function returns `0`
/// or [`Output::rest`] can't be built
/// as a valid UTF-8 string.
/// # Examples
/// ```
/// # use whitehole::{combinator::eater, Combinator};
/// # fn t(_: Combinator!()) {}
/// // accept all the rest characters
/// # t(
/// eater(|input| input.rest().len())
/// # );
/// ```
#[inline]
pub fn eater<State, Heap>(
  f: impl Fn(&mut Input<&mut State, &mut Heap>) -> usize,
) -> Combinator!((), State, Heap) {
  wrap(move |input| match f(input) {
    0 => None,
    digested => input.digest(digested),
  })
}

/// Returns a combinator by the provided function that
/// eats [`Input::rest`] and returns the number of digested bytes.
/// The combinator will reject if the function returns `0`.
/// # Safety
/// You should ensure that [`Output::rest`] can be built
/// as a valid UTF-8 string.
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`eater`].
/// # Examples
/// ```
/// # use whitehole::{combinator::eater_unchecked, Combinator};
/// # fn t(_: Combinator!()) {}
/// // accept all the rest characters
/// # t(
/// unsafe { eater_unchecked(|input| input.rest().len()) }
/// # );
/// ```
#[inline]
pub unsafe fn eater_unchecked<State, Heap>(
  f: impl Fn(&mut Input<&mut State, &mut Heap>) -> usize,
) -> Combinator!((), State, Heap) {
  wrap(move |input| match f(input) {
    0 => None,
    digested => input.digest_unchecked(digested).into(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::action::Action;

  #[test]
  fn combinator_eat() {
    // normal usize
    assert_eq!(
      eat(3)
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // normal str
    assert_eq!(
      eat("123")
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // normal String
    assert_eq!(
      eat("123".to_string())
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // normal char
    assert_eq!(
      eat(';')
        .exec(&mut Input::new(";", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // overflow
    assert_eq!(
      eat(3)
        .exec(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
    // reject
    assert!(eat("123")
      .exec(&mut Input::new("abc", 0, &mut (), &mut ()).unwrap())
      .is_none());
    assert!(eat('1')
      .exec(&mut Input::new("abc", 0, &mut (), &mut ()).unwrap())
      .is_none());
    // invalid code point
    assert_eq!(
      eat(1)
        .exec(&mut Input::new("好", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
    // 0 is allowed
    assert_eq!(
      eat(0)
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("123")
    );
    // empty is allowed
    assert_eq!(
      eat("")
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("123")
    );
  }

  #[test]
  fn combinator_eat_unchecked() {
    // normal
    assert_eq!(
      unsafe { eat_unchecked(3) }
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // 0
    assert_eq!(
      unsafe { eat_unchecked(0) }
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("123")
    );
  }

  #[test]
  #[should_panic]
  fn combinator_eat_unchecked_overflow() {
    unsafe { eat_unchecked(3) }.exec(&mut Input::new("12", 0, &mut (), &mut ()).unwrap());
  }

  #[test]
  #[should_panic]
  fn combinator_eat_unchecked_invalid_code_point() {
    unsafe { eat_unchecked(1) }.exec(&mut Input::new("好", 0, &mut (), &mut ()).unwrap());
  }

  #[test]
  fn combinator_eater() {
    // normal
    assert_eq!(
      eater(|input| input.rest().len())
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // overflow
    assert_eq!(
      eater(|input| input.rest().len() + 1)
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
    // invalid code point
    assert_eq!(
      eater(|_| 1)
        .exec(&mut Input::new("好", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
    // 0
    assert_eq!(
      eater(|_| 0)
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
  }

  #[test]
  fn combinator_eater_unchecked() {
    // normal
    assert_eq!(
      unsafe { eater_unchecked(|input| input.rest().len()) }
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // 0
    assert_eq!(
      unsafe { eater_unchecked(|_| 0) }
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
  }

  #[test]
  #[should_panic]
  fn combinator_eater_unchecked_overflow() {
    unsafe { eater_unchecked(|input| input.rest().len() + 1) }
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap());
  }

  #[test]
  #[should_panic]
  fn combinator_eater_unchecked_invalid_code_point() {
    unsafe { eater_unchecked(|_| 1) }.exec(&mut Input::new("好", 0, &mut (), &mut ()).unwrap());
  }
}
