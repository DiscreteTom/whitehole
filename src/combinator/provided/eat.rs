use crate::{
  action::{Action, Context},
  combinator::{create_value_combinator, Combinator, Output},
  instant::Instant,
};

create_value_combinator!(Eat, "See [`eat`].");

unsafe impl<State, Heap> Action<str, State, Heap> for Eat<char> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: &Instant<&str>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { instant.accept_unchecked(self.inner.len_utf8()) })
  }
}

unsafe impl<State, Heap> Action<str, State, Heap> for Eat<String> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: &Instant<&str>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(&self.inner)
      .then(|| unsafe { instant.accept_unchecked(self.inner.len()) })
  }
}

unsafe impl<State, Heap> Action<str, State, Heap> for Eat<&str> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: &Instant<&str>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { instant.accept_unchecked(self.inner.len()) })
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Eat<u8> {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&[u8]>,
    _: Context<&mut State, &mut Heap>,
  ) -> Option<Output<()>> {
    instant
      .rest()
      .first()
      .is_some_and(|&c| c == self.inner)
      .then(|| unsafe { instant.accept_unchecked(1) })
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Eat<&[u8]> {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&[u8]>,
    _: Context<&mut State, &mut Heap>,
  ) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { instant.accept_unchecked(self.inner.len()) })
  }
}

unsafe impl<const N: usize, State, Heap> Action<[u8], State, Heap> for Eat<&[u8; N]> {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&[u8]>,
    _: Context<&mut State, &mut Heap>,
  ) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { instant.accept_unchecked(N) })
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Eat<Vec<u8>> {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&[u8]>,
    _: Context<&mut State, &mut Heap>,
  ) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(&self.inner)
      .then(|| unsafe { instant.accept_unchecked(self.inner.len()) })
  }
}

/// Returns a combinator to eat from the head of [`Instant::rest`] by the provided pattern.
/// The combinator will reject if the pattern is not found.
/// # Caveats
/// Empty patterns are allowed and will always accept 0 bytes,
/// even when [`Instant::rest`] is empty.
/// Be careful with infinite loops.
/// # Examples
/// For string (`&str`):
/// ```
/// # use whitehole::{combinator::{eat, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action>) {}
/// # t(
/// eat('a') // eat by char
/// # );
/// # t(
/// eat("true") // eat by &str
/// # );
/// # t(
/// eat("true".to_string()) // eat by String
/// # );
/// ```
/// For bytes (`&[u8]`):
/// ```
/// # use whitehole::{combinator::{eat, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action<[u8]>>) {}
/// # t(
/// eat(b'a') // eat by a byte (u8)
/// # );
/// # t(
/// eat(b"true") // eat by &[u8] or &[u8; N]
/// # );
/// # t(
/// eat(vec![b'a']) // eat by Vec<u8>
/// # );
/// ```
#[inline]
pub const fn eat<T>(pattern: T) -> Combinator<Eat<T>> {
  Combinator::new(Eat::new(pattern))
}

macro_rules! impl_into_eat_combinator {
  ($inner:ty) => {
    impl From<$inner> for Combinator<Eat<$inner>> {
      #[inline]
      fn from(v: $inner) -> Self {
        eat(v)
      }
    }
  };
}
impl_into_eat_combinator!(char);
impl_into_eat_combinator!(String);
impl_into_eat_combinator!(u8);
impl_into_eat_combinator!(Vec<u8>);
impl<'a> From<&'a str> for Combinator<Eat<&'a str>> {
  #[inline]
  fn from(v: &'a str) -> Self {
    eat(v)
  }
}
impl<'a> From<&'a [u8]> for Combinator<Eat<&'a [u8]>> {
  #[inline]
  fn from(v: &'a [u8]) -> Self {
    eat(v)
  }
}
impl<'a, const N: usize> From<&'a [u8; N]> for Combinator<Eat<&'a [u8; N]>> {
  #[inline]
  fn from(v: &'a [u8; N]) -> Self {
    eat(v)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{action::Action, digest::Digest, instant::Instant};
  use std::{ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text, Value = ()>,
    input: &Text,
    digested: Option<usize>,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action
        .exec(
          &Instant::new(input),
          Context {
            state: &mut (),
            heap: &mut ()
          }
        )
        .map(|o| o.digested),
      digested
    )
  }

  #[test]
  fn combinator_eat() {
    // normal char
    helper(eat(';'), ";", Some(1));
    // normal &str
    helper(eat("123"), "123", Some(3));
    // normal String
    helper(eat("123".to_string()), "123", Some(3));
    // normal u8
    helper(eat(b';'), b";", Some(1));
    // normal &[u8;N]
    helper(eat(b";"), b";", Some(1));
    // normal &[u8]
    helper(eat("123".as_bytes()), b"123", Some(3));
    // normal Vec<u8>
    helper(eat(vec![b'1', b'2', b'3']), b"123", Some(3));
    // reject
    helper(eat("123"), "abc", None);
    helper(eat('1'), "abc", None);
    // empty string is allowed and always accept
    helper(eat(""), "123", Some(0));
    helper(eat(""), "", Some(0));
    helper(eat("".to_string()), "123", Some(0));
    helper(eat("".to_string()), "", Some(0));
    helper(eat(b""), b"123", Some(0));
    helper(eat(b""), b"", Some(0));
    helper(eat(b"" as &[u8]), b"123", Some(0));
    helper(eat(b"" as &[u8]), b"", Some(0));
    helper(eat(vec![]), b"123", Some(0));
    helper(eat(vec![]), b"", Some(0));
  }

  #[test]
  fn eat_into_combinator() {
    fn test(c: Combinator<impl Action<Value = ()>>) {
      helper(c, "a", Some(1));
    }
    fn test_bytes(c: Combinator<impl Action<[u8], Value = ()>>) {
      helper(c, b"a", Some(1));
    }
    test('a'.into());
    test("a".into());
    test("a".to_string().into());
    test_bytes(b'a'.into());
    test_bytes(b"a".into());
    test_bytes("a".as_bytes().into());
    test_bytes(vec![b'a'].into());
  }
}
