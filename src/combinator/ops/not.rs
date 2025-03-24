//! Overload `!` operator for [`Combinator`].
//!
//! `!Combinator` will create a new combinator
//! that will accept with zero digested if the original combinator rejects,
//! or reject if the original combinator accepts.
//! # Basics
//! ```
//! # use whitehole::{combinator::{eat, take, Combinator}, action::Action};
//! # fn t(_: Combinator<impl Action>) {}
//! # fn tb(_: Combinator<impl Action<[u8]>>) {}
//! // reject if the next char is 'a', otherwise accept with 0 digested
//! // (negative lookahead)
//! # t(
//! !eat('a')
//! # );
//! // apply twice to realize positive lookahead
//! # t(
//! !!eat('a')
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

unsafe impl<Text: ?Sized, State, Heap, T: Action<Text, State, Heap, Value: Default>>
  Action<Text, State, Heap> for Not<T>
{
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    mut ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    if let Some(_) = self.action.exec(instant, ctx.reborrow()) {
      None
    } else {
      Some(Output {
        digested: 0,
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
    helper(!reject(), "1", Some(0));
    helper(!!accept(), "1", Some(0));
    helper(!!accept0(), "1", Some(0));
    helper(!!reject(), "1", None);

    helper(!accept(), "好", None);
    helper(!accept0(), "好", None);
    helper(!reject(), "好", Some(0));
    helper(!!accept(), "好", Some(0));
    helper(!!accept0(), "好", Some(0));
    helper(!!reject(), "好", None);

    helper(!accept(), "", Some(0));
    helper(!accept0(), "", None);
    helper(!reject(), "", Some(0));
    helper(!!accept(), "", None);
    helper(!!accept0(), "", Some(0));
    helper(!!reject(), "", None);

    helper(!accept(), b"1" as &[u8], None);
    helper(!accept0(), b"1" as &[u8], None);
    helper(!reject_b(), b"1" as &[u8], Some(0));
    helper(!!accept(), b"1" as &[u8], Some(0));
    helper(!!accept0(), b"1" as &[u8], Some(0));
    helper(!!reject_b(), b"1" as &[u8], None);

    helper(!accept(), b"" as &[u8], Some(0));
    helper(!accept0(), b"" as &[u8], None);
    helper(!reject_b(), b"" as &[u8], Some(0));
    helper(!!accept(), b"" as &[u8], None);
    helper(!!accept0(), b"" as &[u8], Some(0));
    helper(!!reject_b(), b"" as &[u8], None);

    helper(!eat('a'), "a", None);
    helper(!eat('a'), "b", Some(0));
    helper(!(eat('a') | eat('b')), "a", None);
    helper(!(eat('a') | eat('b')), "b", None);
    helper(!(eat('a') | eat('b')), "c", Some(0));
    helper(!eat('a') + take(1), "a", None);
    helper(!eat('a') + take(1), "b", Some(1));
    helper(!eat('a'), "a", None);
    helper(!!eat('a'), "b", None);
    helper(!!(eat('a') | eat('b')), "a", Some(0));
    helper(!!(eat('a') | eat('b')), "b", Some(0));
    helper(!!(eat('a') | eat('b')), "c", None);
    helper(!!eat('a') + take(1), "a", Some(1));
    helper(!!eat('a') + take(1), "b", None);
  }
}
