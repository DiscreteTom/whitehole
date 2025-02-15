use crate::{
  action::{Action, Context},
  combinator::{create_value_combinator, Combinator, Output},
  instant::Instant,
};

create_value_combinator!(Eat, "See [`eat`].");

unsafe impl<State, Heap> Action<str, State, Heap> for Eat<char> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: Instant<&str>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { instant.accept_unchecked(self.inner.len_utf8()) })
  }
}

unsafe impl<State, Heap> Action<str, State, Heap> for Eat<String> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: Instant<&str>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(&self.inner)
      .then(|| unsafe { instant.accept_unchecked(self.inner.len()) })
  }
}

unsafe impl<State, Heap> Action<str, State, Heap> for Eat<&str> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: Instant<&str>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { instant.accept_unchecked(self.inner.len()) })
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Eat<u8> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: Instant<&[u8]>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
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
  fn exec(&self, instant: Instant<&[u8]>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { instant.accept_unchecked(self.inner.len()) })
  }
}

unsafe impl<const N: usize, State, Heap> Action<[u8], State, Heap> for Eat<&[u8; N]> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: Instant<&[u8]>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { instant.accept_unchecked(N) })
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Eat<Vec<u8>> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: Instant<&[u8]>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .starts_with(&self.inner)
      .then(|| unsafe { instant.accept_unchecked(self.inner.len()) })
  }
}

/// Returns a combinator to eat from the head of [`Instant::rest`](crate::instant::Instant::rest) by the provided pattern.
/// The combinator will reject if the pattern is not found.
///
/// `""` (empty string) is allowed but be careful with infinite loops.
///
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
      fn from(v: $inner) -> Combinator<Eat<$inner>> {
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
  fn from(v: &str) -> Combinator<Eat<&str>> {
    eat(v)
  }
}
impl<'a> From<&'a [u8]> for Combinator<Eat<&'a [u8]>> {
  #[inline]
  fn from(v: &[u8]) -> Combinator<Eat<&[u8]>> {
    eat(v)
  }
}
impl<'a, const N: usize> From<&'a [u8; N]> for Combinator<Eat<&'a [u8; N]>> {
  #[inline]
  fn from(v: &[u8; N]) -> Combinator<Eat<&[u8; N]>> {
    eat(v)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{action::Action, instant::Instant};

  #[test]
  fn combinator_eat() {
    // normal char
    assert_eq!(
      eat(';')
        .exec(Instant::new(";"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
    // normal &str
    assert_eq!(
      eat("123")
        .exec(Instant::new("123"), Context::default())
        .map(|output| output.digested),
      Some(3)
    );
    // normal String
    assert_eq!(
      eat("123".to_string())
        .exec(Instant::new("123"), Context::default())
        .map(|output| output.digested),
      Some(3)
    );
    // normal u8
    assert_eq!(
      eat(b';')
        .exec(Instant::new(b";"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
    // normal &[u8;N]
    assert_eq!(
      eat(b";")
        .exec(Instant::new(b";"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
    // normal &[u8]
    assert_eq!(
      eat("123".as_bytes())
        .exec(Instant::new(b"123"), Context::default())
        .map(|output| output.digested),
      Some(3)
    );
    // normal Vec<u8>
    assert_eq!(
      eat(vec![b'1', b'2', b'3'])
        .exec(Instant::new(b"123"), Context::default())
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(eat("123")
      .exec(Instant::new("abc"), Context::default())
      .is_none());
    assert!(eat('1')
      .exec(Instant::new("abc"), Context::default())
      .is_none());
    // empty string is allowed and always accept
    assert_eq!(
      eat("")
        .exec(Instant::new("123"), Context::default())
        .map(|output| output.digested),
      Some(0)
    );
    assert_eq!(
      eat("")
        .exec(Instant::new(""), Context::default())
        .map(|output| output.digested),
      Some(0)
    );
  }

  #[test]
  fn eat_into_combinator() {
    fn test(c: Combinator<impl Action>) {
      c.exec(Instant::new("a"), Context::default());
    }
    fn test_bytes(c: Combinator<impl Action<[u8]>>) {
      c.exec(Instant::new(b"a"), Context::default());
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
