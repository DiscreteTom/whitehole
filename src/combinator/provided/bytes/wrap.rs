use crate::{
  action::{Action, Input, Output},
  combinator::{create_closure_combinator, Combinator},
  digest::Digest,
  instant::Instant,
};

create_closure_combinator!(
  WrapUnchecked,
  "See [`wrap_unchecked`] and [`wrap_unchecked`]."
);
create_closure_combinator!(Wrap, "See [`wrap`] and [`wrap`].");

macro_rules! impl_wrap {
  ($name:ident, $assert:ident, $text:ty) => {
    unsafe impl<Value, F: Fn(&Instant<&$text>) -> Option<Output<Value>>> Action for $name<F> {
      type Text = $text;
      type State = ();
      type Heap = ();
      type Value = Value;

      #[inline]
      fn exec(
        &self,
        input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
      ) -> Option<Output<Self::Value>> {
        let output = (self.inner)(input.instant);
        $assert!(output.as_ref().map_or(true, |output| input
          .instant
          .rest()
          .validate(output.digested)));
        output
      }
    }
  };
}

impl_wrap!(WrapUnchecked, debug_assert, [u8]);
impl_wrap!(Wrap, assert, [u8]);

/// Wrap a closure or function to create a [`Combinator`] for bytes.
///
/// For the string version, see [`wrap_unchecked`](super::wrap_unchecked).
/// # Safety
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`].
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`wrap`].
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap_unchecked, Combinator};
/// # use whitehole::action::{Output, Action};
/// # fn t() -> Combinator<impl Action<[u8]>> {
/// // eat the next byte if it exists
/// unsafe { wrap_unchecked(|instant| instant.accept(1)) }
/// # }
/// ```
#[inline]
pub const unsafe fn wrap_unchecked<Value, F: Fn(&Instant<&[u8]>) -> Option<Output<Value>>>(
  f: F,
) -> Combinator<WrapUnchecked<F>> {
  Combinator::new(WrapUnchecked::new(f))
}

// TODO: merge dup code

/// Wrap a closure or function to create a [`Combinator`] for bytes.
///
/// For the string version, see [`wrap`](super::wrap).
/// # Panics
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`],
/// otherwise the combinator will panic when executed.
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap, Combinator};
/// # use whitehole::action::{Output, Action};
/// # fn t() -> Combinator<impl Action<[u8]>> {
/// // eat the next byte if it exists
/// wrap(|instant| instant.accept(1))
/// # }
/// ```
#[inline]
pub const fn wrap<Value, F: Fn(&Instant<&[u8]>) -> Option<Output<Value>>>(
  f: F,
) -> Combinator<Wrap<F>> {
  Combinator::new(Wrap::new(f))
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
    let c = unsafe { wrap_unchecked(|instant| instant.accept(1)) };
    helper(c, b"1", 1);

    // ensure the combinator is copyable and clone-able
    let _c = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: WrapUnchecked }");
  }

  #[test]
  #[should_panic]
  fn combinator_bytes_wrap_unchecked_overflow() {
    helper(
      unsafe { wrap_unchecked(|instant| instant.accept_unchecked(4).into()) },
      b"1",
      0,
    );
  }

  #[test]
  fn combinator_bytes_wrap() {
    let c = wrap(|instant| instant.accept(1));
    helper(c, b"1", 1);

    // ensure the combinator is copyable and clone-able
    let _c = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Wrap }");
  }

  #[test]
  #[should_panic]
  fn combinator_bytes_wrap_overflow() {
    helper(
      wrap(|instant| unsafe { instant.accept_unchecked(4) }.into()),
      b"1",
      0,
    );
  }
}
