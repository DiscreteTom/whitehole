use crate::{
  action::{Action, Input},
  combinator::{create_value_combinator, Combinator, Output},
  instant::Instant,
};

create_value_combinator!(Eat, "See [`eat`].");

unsafe impl Action for Eat<char> {
  type Text = str;
  type State = ();
  type Heap = ();
  type Value = ();

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<()>> {
    input
      .instant
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { input.instant.accept_unchecked(self.inner.len_utf8()) })
  }
}

unsafe impl Action for Eat<String> {
  type Text = str;
  type State = ();
  type Heap = ();
  type Value = ();

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<()>> {
    input
      .instant
      .rest()
      .starts_with(&self.inner)
      .then(|| unsafe { input.instant.accept_unchecked(self.inner.len()) })
  }
}

unsafe impl Action for Eat<&str> {
  type Text = str;
  type State = ();
  type Heap = ();
  type Value = ();

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<()>> {
    input
      .instant
      .rest()
      .starts_with(self.inner)
      .then(|| unsafe { input.instant.accept_unchecked(self.inner.len()) })
  }
}

/// Returns a combinator to eat from the head of [`Instant::rest`] by the provided pattern.
/// The combinator will reject if the pattern is not found.
/// # Caveats
/// Empty patterns are allowed and will always accept 0 bytes,
/// even when [`Instant::rest`] is empty.
/// Be careful with infinite loops.
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
pub(super) use impl_into_eat_combinator;
impl_into_eat_combinator!(char);
impl_into_eat_combinator!(String);

impl<'a> From<&'a str> for Combinator<Eat<&'a str>> {
  #[inline]
  fn from(v: &'a str) -> Self {
    eat(v)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{action::Action, digest::Digest, instant::Instant};
  use std::{ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text = Text, Value = (), State = (), Heap = ()>,
    input: &Text,
    digested: Option<usize>,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action
        .exec(Input {
          instant: &Instant::new(input),
          state: &mut (),
          heap: &mut ()
        })
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
    // reject
    helper(eat("123"), "abc", None);
    helper(eat('1'), "abc", None);
    // empty string is allowed and always accept
    helper(eat(""), "123", Some(0));
    helper(eat(""), "", Some(0));
    helper(eat("".to_string()), "123", Some(0));
    helper(eat("".to_string()), "", Some(0));
  }

  #[test]
  fn eat_into_combinator() {
    fn test(c: Combinator<impl Action<Text = str, State = (), Heap = (), Value = ()>>) {
      helper(c, "a", Some(1));
    }
    test('a'.into());
    test("a".into());
    test("a".to_string().into());
  }

  fn _eat_debug() {
    let _ = format!("{:?}", eat('a'));
  }

  fn _eat_clone_copy() {
    let c = eat('a');
    let _c = c;
    let _c = c.clone();
  }
}
