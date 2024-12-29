use crate::{
  action::Action,
  combinator::{wrap_unchecked, Combinator, Input, Output},
  C,
};
use std::marker::PhantomData;

macro_rules! impl_eat {
  ($name:ident, $inner:ty, ($($derive:ident),*)) => {
    /// An [`Action`] implementor.
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

    impl<State, Heap> Into<Combinator<$name<State, Heap>>> for $inner {
      #[inline]
      fn into(self) -> Combinator<$name<State, Heap>> {
        Combinator::new($name::new(self))
      }
    }
  };
}

impl_eat!(EatChar, char, (Copy));
impl_eat!(EatString, String, ());
impl_eat!(EatUsize, usize, (Copy));

unsafe impl<State, Heap> Action for EatChar<State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec<'text>(&self, input: Input<'text, &mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { input.digest_unchecked(self.inner.len_utf8()) })
  }
}

unsafe impl<State, Heap> Action for EatString<State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec<'text>(&self, input: Input<'text, &mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .rest()
      .starts_with(&self.inner)
      .then(|| unsafe { input.digest_unchecked(self.inner.len()) })
  }
}

unsafe impl<State, Heap> Action for EatUsize<State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec<'text>(&self, input: Input<'text, &mut State, &mut Heap>) -> Option<Output<()>> {
    // if eat 1 char, just eat the `input.next` which always exists
    if self.inner == 1 {
      return unsafe { input.digest_unchecked(input.next().len_utf8()) }.into();
    }

    let mut digested: usize = 0;
    let mut count: usize = 0;
    let mut chars = input.rest().chars();
    while count < self.inner {
      // no enough chars, try to digest more
      if let Some(c) = chars.next() {
        digested = unsafe { digested.unchecked_add(c.len_utf8()) };
        // SAFETY: count is always smaller than self.inner which is a usize
        count = unsafe { count.unchecked_add(1) };
      } else {
        // no enough chars, reject
        return None;
      }
    }
    // enough chars
    unsafe { input.digest_unchecked(digested) }.into()
  }
}

/// An [`Action`] implementor.
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

unsafe impl<State, Heap> Action for EatStr<'_, State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec<'text>(&self, input: Input<'text, &mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .rest()
      .starts_with(self.s)
      .then(|| unsafe { input.digest_unchecked(self.s.len()) })
  }
}

impl<'a, State, Heap> Into<Combinator<EatStr<'a, State, Heap>>> for &'a str {
  #[inline]
  fn into(self) -> Combinator<EatStr<'a, State, Heap>> {
    Combinator::new(EatStr::new(self))
  }
}

/// Returns a combinator to eat from the head of [`Input::rest`] by the provided pattern.
/// The combinator will reject if the pattern is not found.
///
/// `0` and `""` (empty string) are allowed but be careful with infinite loops.
///
/// # Examples
/// ```
/// # use whitehole::{combinator::eat, C};
/// # fn t(_: C!()) {}
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
/// eat(10) // eat by char count (not byte length)
/// # );
/// ```
#[inline]
pub fn eat<T>(pattern: impl Into<Combinator<T>>) -> Combinator<T> {
  pattern.into()
}

/// Returns a combinator to eat `n` bytes (not chars) from the head of [`Input::rest`],
/// without checking `n`.
/// The combinator will never reject.
///
/// `0` is allowed but be careful with infinite loops.
/// # Safety
/// You should ensure that the [`Output::digested`] is valid.
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`eat`].
/// # Examples
/// ```
/// # use whitehole::{combinator::eat_unchecked, C};
/// # fn t(_: C!()) {}
/// // eat 10 bytes
/// # t(
/// unsafe { eat_unchecked(10) }
/// # );
/// ```
#[inline]
pub const unsafe fn eat_unchecked<State, Heap>(n: usize) -> C!((), State, Heap) {
  wrap_unchecked(move |input| input.digest_unchecked(n).into())
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
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // normal str
    assert_eq!(
      eat("123")
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // normal String
    assert_eq!(
      eat("123".to_string())
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // normal char
    assert_eq!(
      eat(';')
        .exec(Input::new(";", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(1)
    );
    // overflow
    assert_eq!(
      eat(3)
        .exec(Input::new("12", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // reject
    assert!(eat("123")
      .exec(Input::new("abc", 0, &mut (), &mut ()).unwrap())
      .is_none());
    assert!(eat('1')
      .exec(Input::new("abc", 0, &mut (), &mut ()).unwrap())
      .is_none());
    // 0 is allowed and always accept
    assert_eq!(
      eat(0)
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(0)
    );
    // empty string is allowed and always accept
    assert_eq!(
      eat("")
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(0)
    );
    // eat by chars not bytes
    assert_eq!(
      eat(1)
        .exec(Input::new("好", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    assert_eq!(
      eat(2)
        .exec(Input::new("好好", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(6)
    );
  }

  #[test]
  fn combinator_eat_unchecked() {
    // normal
    assert_eq!(
      unsafe { eat_unchecked(3) }
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // 0
    assert_eq!(
      unsafe { eat_unchecked(0) }
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(0)
    );
  }

  #[test]
  #[should_panic]
  fn combinator_eat_unchecked_overflow() {
    unsafe { eat_unchecked(3) }.exec(Input::new("12", 0, &mut (), &mut ()).unwrap());
  }

  #[test]
  #[should_panic]
  fn combinator_eat_unchecked_invalid_code_point() {
    unsafe { eat_unchecked(1) }.exec(Input::new("好", 0, &mut (), &mut ()).unwrap());
  }
}
