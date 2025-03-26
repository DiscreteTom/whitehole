use crate::{
  action::{Action, Context, Output},
  combinator::Combinator,
  instant::Instant,
};

/// See [`take`].
#[derive(Copy, Clone, Debug)]
pub struct Take {
  n: usize,
}

impl Take {
  #[inline]
  const fn new(n: usize) -> Self {
    Self { n }
  }
}

unsafe impl Action<str> for Take {
  type Value = ();
  type State = ();
  type Heap = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&str>,
    _: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<()>> {
    let mut digested: usize = 0;
    let mut count: usize = 0;
    let mut chars = instant.rest().chars();
    while count < self.n {
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
    unsafe { instant.accept_unchecked(digested) }.into()
  }
}

unsafe impl Action<[u8]> for Take {
  type Value = ();
  type State = ();
  type Heap = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&[u8]>,
    _: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<()>> {
    instant.accept(self.n)
  }
}

/// Returns a combinator to take the next `n` undigested [`char`]s or bytes.
///
/// `0` is allowed but be careful with infinite loops.
/// # Examples
/// For string (`&str`):
/// ```
/// # use whitehole::{combinator::{take, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action>) {}
/// # t(
/// take(10) // take 10 chars
/// # );
/// ```
/// For bytes (`&[u8]`):
/// ```
/// # use whitehole::{combinator::{take, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action<[u8]>>) {}
/// # t(
/// take(10) // take 10 bytes
/// # );
/// ```
#[inline]
pub const fn take(n: usize) -> Combinator<Take> {
  Combinator::new(Take::new(n))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{digest::Digest, instant::Instant};
  use std::{ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text, State = (), Heap = (), Value = ()>,
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
  fn test_take() {
    // normal
    helper(take(3), "123456", Some(3));
    helper(take(3), b"123456" as &[u8], Some(3));
    // reject
    helper(take(7), "123456", None);
    // 0 is always accepted
    helper(take(0), "", Some(0));
    helper(take(0), "123456", Some(0));
    // take by chars not bytes for &str
    helper(take(1), "好", Some(3));
    helper(take(2), "好好", Some(6));
  }
}
