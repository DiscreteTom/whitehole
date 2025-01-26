use crate::{
  action::{Action, Input, Output},
  combinator::{create_closure_combinator, Combinator},
};

create_closure_combinator!(Next, "See [`next`].");

unsafe impl<State, Heap, F: Fn(char) -> bool> Action<str, State, Heap> for Next<F> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    let next = input.instant().rest().chars().next()?;
    if !(self.inner)(next) {
      return None;
    }
    Some(unsafe { input.digest_unchecked(next.len_utf8()) })
  }
}

unsafe impl<State, Heap, F: Fn(u8) -> bool> Action<[u8], State, Heap> for Next<F> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&[u8], &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    let &next = input.instant().rest().first()?;
    if !(self.inner)(next) {
      return None;
    }
    Some(unsafe { input.digest_unchecked(1) })
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
  use crate::{
    action::{Action, Input},
    instant::Instant,
  };

  #[test]
  fn combinator_next() {
    // normal
    assert_eq!(
      next(|c| c.is_ascii_digit())
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(1)
    );
    // reject
    assert!(next(|c| c.is_ascii_alphabetic())
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
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
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(next(|c| c.is_ascii_digit())
      .exec(Input::new(Instant::new("abc"), &mut (), &mut ()))
      .is_none());
  }

  #[test]
  fn combinator_next_bytes() {
    // normal
    assert_eq!(
      bytes::next(|b| b.is_ascii_digit())
        .exec(Input::new(Instant::new(b"123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(1)
    );
    // reject
    assert!(bytes::next(|b| b.is_ascii_alphabetic())
      .exec(Input::new(Instant::new(b"123"), &mut (), &mut ()))
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
        .exec(Input::new(Instant::new(b"123"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(bytes::next(|b| b.is_ascii_digit())
      .exec(Input::new(Instant::new(b"abc"), &mut (), &mut ()))
      .is_none());
  }
}
