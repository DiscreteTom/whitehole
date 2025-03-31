use crate::{
  action::{Action, Input, Output},
  combinator::{provided::create_closure_combinator, Combinator},
  instant::Instant,
};

create_closure_combinator!(Next, "See [`next`].");

unsafe impl<F: Fn(char) -> bool> Action for Next<F> {
  type Text = str;
  type State = ();
  type Heap = ();
  type Value = ();

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let next = input.instant.rest().chars().next()?;
    if !(self.inner)(next) {
      return None;
    }
    Some(unsafe { input.instant.accept_unchecked(next.len_utf8()) })
  }
}

/// Returns a combinator to match the next undigested [`char`] by the condition.
/// The combinator will reject if not matched.
/// # Examples
/// ```
/// # use whitehole::{combinator::{next, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action>) {}
/// // match one ascii digit
/// # t(
/// next(|c| c.is_ascii_digit())
/// # );
/// // match one or more ascii digit
/// # t(
/// next(|c| c.is_ascii_digit()) * (1..)
/// # );
/// ```
#[inline]
pub const fn next<F: Fn(char) -> bool>(condition: F) -> Combinator<Next<F>> {
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
  fn combinator_next() {
    // normal
    helper(next(|c| c.is_ascii_digit()), "123", Some(1));
    // reject
    helper(next(|c| c.is_ascii_alphabetic()), "123", None);
    // utf8
    helper(next(|_| true), "好", Some(3));

    // ensure the combinator is copyable and clone-able
    let c = next(|c| c.is_ascii_digit());
    let _c = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Next }");
  }

  #[test]
  fn one_or_more_next() {
    // normal
    helper(next(|c| c.is_ascii_digit()) * (1..), "123", Some(3));
    // reject
    helper(next(|c| c.is_ascii_digit()) * (1..), "abc", None);
    // utf8
    helper(next(|_| true) * (1..), "好好好", Some(9));
  }
}
