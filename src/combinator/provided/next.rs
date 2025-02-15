use crate::{
  action::{Action, Context, Output},
  combinator::{create_closure_combinator, Combinator},
  instant::Instant,
};

create_closure_combinator!(Next, "See [`next`].");

unsafe impl<State, Heap, F: Fn(char) -> bool> Action<str, State, Heap> for Next<F> {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    instant: Instant<&str>,
    _: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    let next = instant.rest().chars().next()?;
    if !(self.inner)(next) {
      return None;
    }
    Some(unsafe { instant.accept_unchecked(next.len_utf8()) })
  }
}

unsafe impl<State, Heap, F: Fn(u8) -> bool> Action<[u8], State, Heap> for Next<F> {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    instant: Instant<&[u8]>,
    _: Context<&mut State, &mut Heap>,
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
  use crate::{action::Action, instant::Instant};

  #[test]
  fn combinator_next() {
    // normal
    assert_eq!(
      next(|c| c.is_ascii_digit())
        .exec(Instant::new("123"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
    // reject
    assert!(next(|c| c.is_ascii_alphabetic())
      .exec(Instant::new("123"), Context::default())
      .is_none());

    // ensure the combinator is copyable and clone-able
    let c = next(|c| c.is_ascii_digit());
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Next }");
  }

  #[test]
  fn one_or_more_next() {
    // normal
    assert_eq!(
      (next(|c| c.is_ascii_digit()) * (1..))
        .exec(Instant::new("123"), Context::default())
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(next(|c| c.is_ascii_digit())
      .exec(Instant::new("abc"), Context::default())
      .is_none());
  }

  #[test]
  fn combinator_next_bytes() {
    // normal
    assert_eq!(
      bytes::next(|b| b.is_ascii_digit())
        .exec(Instant::new(b"123"), Context::default())
        .map(|output| output.digested),
      Some(1)
    );
    // reject
    assert!(bytes::next(|b| b.is_ascii_alphabetic())
      .exec(Instant::new(b"123"), Context::default())
      .is_none());

    // ensure the combinator is copyable and clone-able
    let c = bytes::next(|b| b.is_ascii_digit());
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Next }");
  }

  #[test]
  fn one_or_more_next_bytes() {
    // normal
    assert_eq!(
      (bytes::next(|b| b.is_ascii_digit()) * (1..))
        .exec(Instant::new(b"123"), Context::default())
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(bytes::next(|b| b.is_ascii_digit())
      .exec(Instant::new(b"abc"), Context::default())
      .is_none());
  }
}
