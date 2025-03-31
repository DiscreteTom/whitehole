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
  /// Create a new instance.
  #[inline]
  pub const fn new(n: usize) -> Self {
    Self { n }
  }
}

unsafe impl Action for Take {
  type Text = str;
  type State = ();
  type Heap = ();
  type Value = ();

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let mut digested: usize = 0;
    let mut chars = input.instant.rest().chars();
    for _ in 0..self.n {
      if let Some(c) = chars.next() {
        digested = unsafe { digested.unchecked_add(c.len_utf8()) };
      } else {
        // no enough chars, reject
        return None;
      }
    }
    // enough chars
    unsafe { input.instant.accept_unchecked(digested) }.into()
  }
}

/// Returns a combinator to take the next `n` undigested [`char`]s.
///
/// `0` is allowed but be careful with infinite loops.
/// # Examples
/// ```
/// # use whitehole::{combinator::{take, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action>) {}
/// # t(
/// take(10) // take 10 chars
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
    helper(take(3), "123456", Some(3));
    // reject
    helper(take(7), "123456", None);
    // 0 is always accepted
    helper(take(0), "", Some(0));
    helper(take(0), "123456", Some(0));
    // take by chars not bytes for &str
    helper(take(1), "好", Some(3));
    helper(take(2), "好好", Some(6));
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
