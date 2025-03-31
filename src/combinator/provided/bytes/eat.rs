use crate::{
  action::{Action, Input},
  combinator::{create_value_combinator, Combinator, Output},
  instant::Instant,
};

create_value_combinator!(Eat, "See [`eat`].");

unsafe impl Action for Eat<u8> {
  type Text = [u8];
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
      .first()
      .is_some_and(|&c| c == self.inner)
      .then(|| unsafe { input.instant.accept_unchecked(1) })
  }
}

unsafe impl Action for Eat<&[u8]> {
  type Text = [u8];
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

unsafe impl<const N: usize> Action for Eat<&[u8; N]> {
  type Text = [u8];
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
      .then(|| unsafe { input.instant.accept_unchecked(N) })
  }
}

unsafe impl Action for Eat<Vec<u8>> {
  type Text = [u8];
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

impl_into_eat_combinator!(u8);
impl_into_eat_combinator!(Vec<u8>);

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
    // normal u8
    helper(eat(b';'), b";", Some(1));
    // normal &[u8;N]
    helper(eat(b";"), b";", Some(1));
    // normal &[u8]
    helper(eat("123".as_bytes()), b"123", Some(3));
    // normal Vec<u8>
    helper(eat(vec![b'1', b'2', b'3']), b"123", Some(3));
    // reject
    helper(eat(b""), b"123", Some(0));
    helper(eat(b""), b"", Some(0));
    helper(eat(b"" as &[u8]), b"123", Some(0));
    helper(eat(b"" as &[u8]), b"", Some(0));
    helper(eat(vec![]), b"123", Some(0));
    helper(eat(vec![]), b"", Some(0));
  }

  #[test]
  fn eat_into_combinator() {
    fn test_bytes(c: Combinator<impl Action<Text = [u8], State = (), Heap = (), Value = ()>>) {
      helper(c, b"a", Some(1));
    }
    test_bytes(b'a'.into());
    test_bytes(b"a".into());
    test_bytes("a".as_bytes().into());
    test_bytes(vec![b'a'].into());
  }

  fn _eat_debug() {
    let _ = format!("{:?}", eat(b'a'));
  }

  fn _eat_clone_copy() {
    let c = eat(b'a');
    let _c = c;
    let _c = c.clone();
  }
}
