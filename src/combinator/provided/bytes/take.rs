use crate::{
  action::{Action, Input, Output},
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
  /// Create a new instance.
  pub const fn new(n: usize) -> Self {
    Self { n }
  }
}

unsafe impl Action for Take {
  type Text = [u8];
  type State = ();
  type Heap = ();
  type Value = ();

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    input.instant.accept(self.n)
  }
}

/// Returns a combinator to take the next `n` undigested or bytes.
///
/// `0` is allowed but be careful with infinite loops.
/// # Examples
/// ```
/// # use whitehole::{combinator::{bytes, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action<Text=[u8]>>) {}
/// # t(
/// bytes::take(10) // take 10 bytes
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
    action: impl Action<Text = Text, State = (), Heap = (), Value = ()>,
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
  fn test_take() {
    // normal
    helper(take(3), b"123456" as &[u8], Some(3));
    // reject
    helper(take(7), b"123456", None);
    // 0 is always accepted
    helper(take(0), b"", Some(0));
    helper(take(0), b"123456", Some(0));
  }

  fn _take_debug() {
    let _ = format!("{:?}", take(0));
  }

  fn _take_clone_copy() {
    let c = take(0);
    let _c = c;
    let _c = c.clone();
  }
}
