use crate::{
  action::{Action, Input, Output},
  combinator::{create_closure_combinator, Combinator},
  instant::Instant,
};

create_closure_combinator!(Next, "See [`next`].");

unsafe impl<F: Fn(u8) -> bool> Action for Next<F> {
  type Text = [u8];
  type State = ();
  type Heap = ();
  type Value = ();

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let &next = input.instant.rest().first()?;
    if !(self.inner)(next) {
      return None;
    }
    Some(unsafe { input.instant.accept_unchecked(1) })
  }
}

/// Returns a combinator to match the next undigested byte by the condition.
/// The combinator will reject if not matched.
/// # Examples
/// ```
/// # use whitehole::{combinator::{next, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action<[u8]>>) {}
/// // match one ascii digit
/// # t(
/// next(|b| b.is_ascii_digit())
/// # );
/// // match one or more ascii digit
/// # t(
/// next(|b| b.is_ascii_digit()) * (1..)
/// # );
/// ```
#[inline]
pub const fn next<F: Fn(u8) -> bool>(condition: F) -> Combinator<Next<F>> {
  Combinator::new(Next::new(condition))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{action::Action, digest::Digest, instant::Instant};
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
  fn combinator_next_bytes() {
    // normal
    helper(next(|b| b.is_ascii_digit()), b"123", Some(1));
    // reject
    helper(next(|b| b.is_ascii_alphabetic()), b"123", None);

    // ensure the combinator is copyable and clone-able
    let c = next(|b| b.is_ascii_digit());
    let _c = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Next }");
  }

  #[test]
  fn one_or_more_next_bytes() {
    // normal
    helper(next(|b| b.is_ascii_digit()) * (1..), b"123", Some(3));
    // reject
    helper(next(|b| b.is_ascii_digit()) * (1..), b"abc", None);
  }
}
