//! Overload `!` operator for [`Combinator`].
//!
//! `!Combinator` will create a new combinator
//! that will accept one byte or [`char`] if the original combinator rejects,
//! or reject if the original combinator accepts.
//! # Basics
//! ```
//! # use whitehole::{combinator::{eat, take, Combinator}, action::Action};
//! # fn t(_: Combinator<impl Action>) {}
//! # fn tb(_: Combinator<impl Action<[u8]>>) {}
//! // match one char that is not 'a'
//! # t(
//! !eat('a')
//! # );
//! // match one char that is not 'a' or 'b'
//! # t(
//! !(eat('a') | eat('b'))
//! # );
//! // accept more than one char with `take`
//! # t(
//! !eat('a') + take(3)
//! # );
//! ```

use crate::{
  action::Context,
  combinator::{Action, Combinator, Output},
  instant::Instant,
};
use std::ops;

/// An [`Action`] created by the `!` operator.
/// See [`ops::not`](crate::combinator::ops::not) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Not<T> {
  action: T,
}

impl<T> Not<T> {
  #[inline]
  const fn new(action: T) -> Self {
    Self { action }
  }
}

unsafe impl<State, Heap, T: Action<[u8], State, Heap, Value: Default>> Action<[u8], State, Heap>
  for Not<T>
{
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&[u8]>,
    mut ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    if let Some(_) = self.action.exec(instant, ctx.reborrow()) {
      None
    } else {
      instant
        .accept(1)
        .map(|output| output.map(|_| Default::default()))
    }
  }
}

unsafe impl<State, Heap, T: Action<str, State, Heap, Value: Default>> Action<str, State, Heap>
  for Not<T>
{
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&str>,
    mut ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    if let Some(_) = self.action.exec(instant, ctx.reborrow()) {
      None
    } else {
      Some(Output {
        digested: instant.rest().chars().next()?.len_utf8(),
        value: Default::default(),
      })
    }
  }
}

impl<T> ops::Not for Combinator<T> {
  type Output = Combinator<Not<T>>;

  /// See [`ops::not`](crate::combinator::ops::not) for more information.
  #[inline]
  fn not(self) -> Self::Output {
    Self::Output::new(Not::new(self.action))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{eat, take},
    digest::Digest,
    instant::Instant,
  };
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
  fn test_not() {
    let accept = || take(1);
    let accept0 = || take(0);
    let reject = || take(1).reject(|_, _| true);
    let reject_b = || take(1).reject(|_, _| true);

    helper(!accept(), "1", None);
    helper(!accept0(), "1", None);
    helper(!reject(), "1", Some(1));

    helper(!accept(), "好", None);
    helper(!accept0(), "好", None);
    helper(!reject(), "好", Some(3));

    helper(!accept(), "", None);
    helper(!accept0(), "", None);
    helper(!reject(), "", None);

    helper(!accept(), b"1" as &[u8], None);
    helper(!accept0(), b"1" as &[u8], None);
    helper(!reject_b(), b"1" as &[u8], Some(1));

    helper(!accept(), b"" as &[u8], None);
    helper(!accept0(), b"" as &[u8], None);
    helper(!reject_b(), b"" as &[u8], None);

    helper(!eat('a'), "a", None);
    helper(!eat('a'), "b", Some(1));
    helper(!(eat('a') | eat('b')), "a", None);
    helper(!(eat('a') | eat('b')), "b", None);
    helper(!(eat('a') | eat('b')), "c", Some(1));
    helper(!eat('a') + take(1), "aa", None);
    helper(!eat('a') + take(1), "b", None);
    helper(!eat('a') + take(1), "ba", Some(2));
  }
}
