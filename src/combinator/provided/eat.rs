use crate::{
  action::Action,
  combinator::{wrap_unchecked, Combinator, Input, Output},
  C,
};
use std::marker::PhantomData;

/// A util trait to make [`eat`] generic over different types.
///
/// Built-in implementations are provided for [`String`], `&str`, [`char`] and [`usize`].
///
/// See [`eat`] for more details.
pub trait Eat<State, Heap> {
  /// Check if the rest of input text starts with this instance.
  /// Return the output after digesting the instance if found.
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>>;
}

macro_rules! impl_eat {
  ($name:ident, $inner:ty, ($($derive:ident),*)) => {
    /// An [`Action`] and [`Eat`] implementor.
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

    impl<State, Heap> From<$inner> for Combinator<$name<State, Heap>> {
      #[inline]
      fn from(v: $inner) -> Combinator<$name<State, Heap>> {
        Combinator::new($name::new(v))
      }
    }

    unsafe impl<State, Heap> Action for $name<State, Heap> {
      type Value = ();
      type State = State;
      type Heap = Heap;

      #[inline]
      fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
        Eat::exec(&self.inner, input)
      }
    }

    impl<State, Heap> Eat<State, Heap> for $name<State, Heap> {
      #[inline]
      fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
        Action::exec(self, input)
      }
    }
  };
}

impl_eat!(EatChar, char, (Copy));
impl_eat!(EatString, String, ());
impl_eat!(EatUsize, usize, (Copy));

impl<State, Heap> Eat<State, Heap> for char {
  #[inline]
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .starts_with(*self)
      .then(|| unsafe { input.digest_unchecked(self.len_utf8()) })
  }
}

impl<State, Heap> Eat<State, Heap> for String {
  #[inline]
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .starts_with(self)
      .then(|| unsafe { input.digest_unchecked(self.len()) })
  }
}

impl<State, Heap> Eat<State, Heap> for usize {
  #[inline]
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
    // if eat 1 char, just eat the `input.next` which always exists
    if *self == 1 {
      return unsafe { input.digest_unchecked(input.next().len_utf8()) }.into();
    }

    let mut digested: usize = 0;
    let mut count: usize = 0;
    let mut chars = input.instant().rest().chars();
    while count < *self {
      // no enough chars, try to digest more
      if let Some(c) = chars.next() {
        digested = unsafe { digested.unchecked_add(c.len_utf8()) };
        // SAFETY: count is always smaller than self which is a usize
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
  inner: &'a str,
  _phantom: PhantomData<(State, Heap)>,
}

impl<'a, State, Heap> EatStr<'a, State, Heap> {
  /// Create a new instance with the inner value.
  #[inline]
  pub const fn new(inner: &'a str) -> Self {
    Self {
      inner,
      _phantom: PhantomData,
    }
  }
}

impl<'a, State, Heap> From<&'a str> for Combinator<EatStr<'a, State, Heap>> {
  #[inline]
  fn from(v: &'a str) -> Combinator<EatStr<'a, State, Heap>> {
    Combinator::new(EatStr::new(v))
  }
}

impl<State, Heap> Eat<State, Heap> for &str {
  #[inline]
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .starts_with(self)
      .then(|| unsafe { input.digest_unchecked(self.len()) })
  }
}

unsafe impl<State, Heap> Action for EatStr<'_, State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
    Eat::exec(&self.inner, input)
  }
}

impl<State, Heap> Eat<State, Heap> for EatStr<'_, State, Heap> {
  #[inline]
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
    Action::exec(self, input)
  }
}

/// Returns a combinator to eat from the head of [`Instant::rest`](crate::instant::Instant::rest) by the provided pattern.
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
pub const fn eat<State, Heap>(pattern: impl Eat<State, Heap>) -> C!((), State, Heap) {
  unsafe { wrap_unchecked(move |input| pattern.exec(input)) }
}

/// Returns a combinator to eat `n` bytes (not chars) from the head of [`Instant::rest`](crate::instant::Instant::rest),
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
  use crate::{action::Action, instant::Instant};

  #[test]
  fn combinator_eat() {
    // normal usize
    assert_eq!(
      eat(3)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // normal str
    assert_eq!(
      eat("123")
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // normal String
    assert_eq!(
      eat("123".to_string())
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // normal char
    assert_eq!(
      eat(';')
        .exec(Input::new(Instant::new(";"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(1)
    );
    // overflow
    assert_eq!(
      eat(3)
        .exec(Input::new(Instant::new("12"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // reject
    assert!(eat("123")
      .exec(Input::new(Instant::new("abc"), &mut (), &mut ()).unwrap())
      .is_none());
    assert!(eat('1')
      .exec(Input::new(Instant::new("abc"), &mut (), &mut ()).unwrap())
      .is_none());
    // 0 is allowed and always accept
    assert_eq!(
      eat(0)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(0)
    );
    // empty string is allowed and always accept
    assert_eq!(
      eat("")
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(0)
    );
    // eat by chars not bytes
    assert_eq!(
      eat(1)
        .exec(Input::new(Instant::new("好"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    assert_eq!(
      eat(2)
        .exec(Input::new(Instant::new("好好"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(6)
    );
  }

  #[test]
  fn combinator_eat_unchecked() {
    // normal
    assert_eq!(
      unsafe { eat_unchecked(3) }
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // 0
    assert_eq!(
      unsafe { eat_unchecked(0) }
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(0)
    );
  }

  #[test]
  #[should_panic]
  fn combinator_eat_unchecked_overflow() {
    unsafe { eat_unchecked(3) }.exec(Input::new(Instant::new("12"), &mut (), &mut ()).unwrap());
  }

  #[test]
  #[should_panic]
  fn combinator_eat_unchecked_invalid_code_point() {
    unsafe { eat_unchecked(1) }.exec(Input::new(Instant::new("好"), &mut (), &mut ()).unwrap());
  }

  #[test]
  fn eat_into_combinator() {
    fn test(c: C!()) {
      c.exec(Input::new(Instant::new("a"), &mut (), &mut ()).unwrap());
    }
    test(1.into());
    test('a'.into());
    test("a".into());
    test("a".to_string().into());
  }

  #[test]
  fn eat_eat() {
    fn test(c: C!()) {
      c.exec(Input::new(Instant::new("a"), &mut (), &mut ()).unwrap());
    }
    test(eat(EatUsize::new(1)));
    test(eat(EatChar::new('a')));
    test(eat(EatStr::new("a")));
    test(eat(EatString::new("a".to_string())));
  }
}
