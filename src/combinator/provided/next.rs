use crate::{
  action::{Action, Context, Output},
  combinator::{create_closure_combinator, Combinator},
  instant::Instant,
};

create_closure_combinator!(Next, "See [`next`].");

unsafe impl<F: Fn(char) -> bool> Action<str> for Next<F> {
  type Value = ();
  type State = ();
  type Heap = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&str>,
    _: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let next = instant.rest().chars().next()?;
    if !(self.inner)(next) {
      return None;
    }
    Some(unsafe { instant.accept_unchecked(next.len_utf8()) })
  }
}

unsafe impl<F: Fn(u8) -> bool> Action<[u8]> for Next<F> {
  type Value = ();
  type State = ();
  type Heap = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&[u8]>,
    _: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let &next = instant.rest().first()?;
    if !(self.inner)(next) {
      return None;
    }
    Some(unsafe { instant.accept_unchecked(1) })
  }
}

/// Returns a combinator to match the next undigested [`char`] by the condition.
/// The combinator will reject if not matched.
///
/// For the bytes version, see [`bytes::next`].
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

pub mod bytes {
  use super::*;

  /// Returns a combinator to match the next undigested byte by the condition.
  /// The combinator will reject if not matched.
  ///
  /// For the string version, see [`next`](super::next).
  /// # Examples
  /// ```
  /// # use whitehole::{combinator::{bytes::next, Combinator}, action::Action};
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{action::Action, digest::Digest, instant::Instant};
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

  #[test]
  fn combinator_next_bytes() {
    // normal
    helper(bytes::next(|b| b.is_ascii_digit()), b"123", Some(1));
    // reject
    helper(bytes::next(|b| b.is_ascii_alphabetic()), b"123", None);

    // ensure the combinator is copyable and clone-able
    let c = bytes::next(|b| b.is_ascii_digit());
    let _c = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Next }");
  }

  #[test]
  fn one_or_more_next_bytes() {
    // normal
    helper(bytes::next(|b| b.is_ascii_digit()) * (1..), b"123", Some(3));
    // reject
    helper(bytes::next(|b| b.is_ascii_digit()) * (1..), b"abc", None);
  }
}
