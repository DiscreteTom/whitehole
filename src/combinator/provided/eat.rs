use crate::{
  action::Action,
  combinator::{create_value_combinator, Combinator, Input, Output},
};

create_value_combinator!(Eat, "See [`eat`].");

unsafe impl<State, Heap> Action<State, Heap> for Eat<char> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { input.digest_unchecked(self.inner.len_utf8()) })
  }
}

unsafe impl<State, Heap> Action<State, Heap> for Eat<String> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .starts_with(&self.inner)
      .then(|| unsafe { input.digest_unchecked(self.inner.len()) })
  }
}

unsafe impl<State, Heap> Action<State, Heap> for Eat<&str> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { input.digest_unchecked(self.inner.len()) })
  }
}

// TODO: remove this
unsafe impl<State, Heap> Action<State, Heap> for Eat<&String> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { input.digest_unchecked(self.inner.len()) })
  }
}

unsafe impl<State, Heap> Action<State, Heap> for Eat<usize> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<()>> {
    let mut digested: usize = 0;
    let mut count: usize = 0;
    let mut chars = input.instant().rest().chars();
    while count < self.inner {
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

/// Returns a combinator to eat from the head of [`Instant::rest`](crate::instant::Instant::rest) by the provided pattern.
/// The combinator will reject if the pattern is not found.
///
/// `0` and `""` (empty string) are allowed but be careful with infinite loops.
///
/// # Examples
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
/// # t(
/// eat(10) // eat by char count (not byte length)
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
impl_into_eat_combinator!(usize);

impl<'a> From<&'a str> for Combinator<Eat<&'a str>> {
  #[inline]
  fn from(v: &str) -> Combinator<Eat<&str>> {
    eat(v)
  }
}
impl<'a> From<&'a String> for Combinator<Eat<&'a String>> {
  #[inline]
  fn from(v: &String) -> Combinator<Eat<&String>> {
    eat(v)
  }
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
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
    // normal &str
    assert_eq!(
      eat("123")
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
    // normal String
    assert_eq!(
      eat("123".to_string())
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
    // normal &String
    assert_eq!(
      eat(&"123".to_string())
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
    // normal char
    assert_eq!(
      eat(';')
        .exec(Input::new(Instant::new(";"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(1)
    );
    // overflow
    assert_eq!(
      eat(3)
        .exec(Input::new(Instant::new("12"), &mut (), &mut ()))
        .map(|output| output.digested),
      None
    );
    // reject
    assert!(eat("123")
      .exec(Input::new(Instant::new("abc"), &mut (), &mut ()))
      .is_none());
    assert!(eat('1')
      .exec(Input::new(Instant::new("abc"), &mut (), &mut ()))
      .is_none());
    // 0 is allowed and always accept
    assert_eq!(
      eat(0)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(0)
    );
    // empty string is allowed and always accept
    assert_eq!(
      eat("")
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(0)
    );
    // eat by chars not bytes
    assert_eq!(
      eat(1)
        .exec(Input::new(Instant::new("好"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
    assert_eq!(
      eat(2)
        .exec(Input::new(Instant::new("好好"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(6)
    );
  }

  #[test]
  fn eat_into_combinator() {
    fn test(c: Combinator<impl Action>) {
      c.exec(Input::new(Instant::new("a"), &mut (), &mut ()));
    }
    test(1.into());
    test('a'.into());
    test("a".into());
    test("a".to_string().into());
    test((&"a".to_string()).into());
  }
}
