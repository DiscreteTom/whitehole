use crate::{
  action::{Action, Input, Output},
  combinator::{
    provided::{create_closure_combinator, impl_wrap},
    Combinator, Contextual,
  },
  digest::Digest,
  instant::Instant,
};

create_closure_combinator!(WrapUnchecked, "See [`wrap_unchecked`].");
create_closure_combinator!(Wrap, "See [`wrap`].");

impl_wrap!(WrapUnchecked, debug_assert, [u8]);
impl_wrap!(Wrap, assert, [u8]);

// TODO: for non-contextual version, can we just return Combinator<Wrap>?

/// Wrap a closure or function to create a [`Combinator`] for bytes.
/// # Safety
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`].
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`wrap`].
/// # Examples
/// ```
/// # use whitehole::combinator::{bytes, Combinator};
/// # use whitehole::action::{Output, Action};
/// # fn t() -> Combinator<impl Action<Text=[u8]>> {
/// // eat the next byte if it exists
/// unsafe { bytes::wrap_unchecked(|input| input.instant.accept(1)) }
/// # }
/// ```
#[inline]
pub const unsafe fn wrap_unchecked<
  Value,
  F: Fn(Input<&Instant<&[u8]>, &mut (), &mut ()>) -> Option<Output<Value>>,
>(
  f: F,
) -> Combinator<Contextual<WrapUnchecked<F>, (), ()>> {
  Combinator::new(Contextual::new(WrapUnchecked::new(f)))
}

/// Wrap a closure or function to create a [`Combinator`] for bytes.
/// # Panics
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`],
/// otherwise the combinator will panic when executed.
/// # Examples
/// ```
/// # use whitehole::combinator::{bytes, Combinator};
/// # use whitehole::action::{Output, Action};
/// # fn t() -> Combinator<impl Action<Text=[u8]>> {
/// // eat the next byte if it exists
/// bytes::wrap(|input| input.instant.accept(1))
/// # }
/// ```
#[inline]
pub const fn wrap<
  Value,
  F: Fn(Input<&Instant<&[u8]>, &mut (), &mut ()>) -> Option<Output<Value>>,
>(
  f: F,
) -> Combinator<Contextual<Wrap<F>, (), ()>> {
  Combinator::new(Contextual::new(Wrap::new(f)))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::instant::Instant;
  use std::{ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text = Text, State = (), Heap = (), Value = ()>,
    input: &Text,
    digested: usize,
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
        .unwrap()
        .digested,
      digested
    )
  }

  #[test]
  fn combinator_bytes_wrap_unchecked() {
    let c = unsafe { wrap_unchecked(|input| input.instant.accept(1)) };
    helper(c, b"1", 1);

    // ensure the combinator is copyable and clone-able
    let _c = c;
    let _c = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(
      format!("{:?}", c),
      "Combinator { action: Contextual(WrapUnchecked) }"
    );
  }

  #[test]
  #[should_panic]
  fn combinator_bytes_wrap_unchecked_overflow() {
    helper(
      unsafe { wrap_unchecked(|input| input.instant.accept_unchecked(4).into()) },
      b"1",
      0,
    );
  }

  #[test]
  fn combinator_bytes_wrap() {
    let c = wrap(|input| input.instant.accept(1));
    helper(c, b"1", 1);

    // ensure the combinator is copyable and clone-able
    let _c = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(
      format!("{:?}", c),
      "Combinator { action: Contextual(Wrap) }"
    );
  }

  #[test]
  #[should_panic]
  fn combinator_bytes_wrap_overflow() {
    helper(
      wrap(|input| unsafe { input.instant.accept_unchecked(4) }.into()),
      b"1",
      0,
    );
  }
}
