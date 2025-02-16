use crate::{
  action::{Action, Context, Output},
  combinator::{create_closure_combinator, Combinator},
  digest::Digest,
  instant::Instant,
};

create_closure_combinator!(
  WrapUnchecked,
  "See [`wrap_unchecked`] and [`bytes::wrap_unchecked`]."
);
create_closure_combinator!(Wrap, "See [`wrap`] and [`bytes::wrap`].");

macro_rules! impl_wrap {
  ($name:ident, $assert:ident, $text:ty) => {
    unsafe impl<
        Value,
        State,
        Heap,
        F: Fn(&Instant<&$text>, Context<&mut State, &mut Heap>) -> Option<Output<Value>>,
      > Action<$text, State, Heap> for $name<F>
    {
      type Value = Value;

      #[inline]
      fn exec(
        &self,
        instant: &Instant<&$text>,
        ctx: Context<&mut State, &mut Heap>,
      ) -> Option<Output<Self::Value>> {
        let output = (self.inner)(instant, ctx);
        $assert!(output
          .as_ref()
          .map_or(true, |output| instant.rest().validate(output.digested)));
        output
      }
    }
  };
}

impl_wrap!(WrapUnchecked, debug_assert, str);
impl_wrap!(Wrap, assert, str);
impl_wrap!(WrapUnchecked, debug_assert, [u8]);
impl_wrap!(Wrap, assert, [u8]);

/// Wrap a closure or function to create a [`Combinator`].
///
/// For the bytes version, see [`bytes::wrap_unchecked`].
/// # Safety
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`].
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`wrap`].
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap_unchecked, Combinator};
/// # use whitehole::action::{Output, Action};
/// # fn t() -> Combinator<impl Action> {
/// // eat the next character if it exists
/// unsafe { wrap_unchecked(|instant, _| instant.rest().chars().next().and_then(|c| instant.accept(c.len_utf8()))) }
/// # }
/// ```
#[inline]
pub const unsafe fn wrap_unchecked<
  State,
  Heap,
  Value,
  F: Fn(&Instant<&str>, Context<&mut State, &mut Heap>) -> Option<Output<Value>>,
>(
  f: F,
) -> Combinator<WrapUnchecked<F>> {
  Combinator::new(WrapUnchecked::new(f))
}

/// Wrap a closure or function to create a [`Combinator`].
///
/// For the bytes version, see [`bytes::wrap`].
/// # Panics
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`],
/// otherwise the combinator will panic when executed.
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap, Combinator};
/// # use whitehole::action::{Output, Action};
/// # fn t() -> Combinator<impl Action> {
/// // eat the next character if it exists
/// wrap(|instant, _| instant.rest().chars().next().and_then(|c| instant.accept(c.len_utf8())))
/// # }
/// ```
#[inline]
pub const fn wrap<
  State,
  Heap,
  Value,
  F: Fn(&Instant<&str>, Context<&mut State, &mut Heap>) -> Option<Output<Value>>,
>(
  f: F,
) -> Combinator<Wrap<F>> {
  Combinator::new(Wrap::new(f))
}

pub mod bytes {
  use super::*;

  /// Wrap a closure or function to create a [`Combinator`] for bytes.
  ///
  /// For the string version, see [`wrap_unchecked`](super::wrap_unchecked).
  /// # Safety
  /// The returned [`Output`] should satisfy the requirement of [`Output::digested`].
  /// This will be checked using [`debug_assert!`].
  /// For the checked version, see [`wrap`].
  /// # Examples
  /// ```
  /// # use whitehole::combinator::{bytes::wrap_unchecked, Combinator};
  /// # use whitehole::action::{Output, Action};
  /// # fn t() -> Combinator<impl Action<[u8]>> {
  /// // eat the next byte if it exists
  /// unsafe { wrap_unchecked(|instant, _| instant.accept(1)) }
  /// # }
  /// ```
  #[inline]
  pub const unsafe fn wrap_unchecked<
    State,
    Heap,
    Value,
    F: Fn(&Instant<&[u8]>, Context<&mut State, &mut Heap>) -> Option<Output<Value>>,
  >(
    f: F,
  ) -> Combinator<WrapUnchecked<F>> {
    Combinator::new(WrapUnchecked::new(f))
  }

  /// Wrap a closure or function to create a [`Combinator`] for bytes.
  ///
  /// For the string version, see [`wrap`](super::wrap).
  /// # Panics
  /// The returned [`Output`] should satisfy the requirement of [`Output::digested`],
  /// otherwise the combinator will panic when executed.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::{bytes::wrap, Combinator};
  /// # use whitehole::action::{Output, Action};
  /// # fn t() -> Combinator<impl Action<[u8]>> {
  /// // eat the next byte if it exists
  /// wrap(|instant, _| instant.accept(1))
  /// # }
  /// ```
  #[inline]
  pub const fn wrap<
    State,
    Heap,
    Value,
    F: Fn(&Instant<&[u8]>, Context<&mut State, &mut Heap>) -> Option<Output<Value>>,
  >(
    f: F,
  ) -> Combinator<Wrap<F>> {
    Combinator::new(Wrap::new(f))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::instant::Instant;
  use std::{ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text, Value = ()>,
    input: &Text,
    digested: usize,
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
        .unwrap()
        .digested,
      digested
    )
  }

  #[test]
  fn combinator_wrap_unchecked() {
    let c = unsafe { wrap_unchecked(|instant, _| instant.accept(1)) };
    helper(c, "1", 1);

    // ensure the combinator is copyable and clone-able
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: WrapUnchecked }");
  }

  #[test]
  fn combinator_wrap_unchecked_fn() {
    fn action(instant: &Instant<&str>, _: Context<&mut (), &mut ()>) -> Option<Output<()>> {
      instant.accept(1)
    }
    let c = unsafe { wrap_unchecked(action) };
    helper(c, "1", 1);

    // ensure the combinator is copyable and clone-able
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: WrapUnchecked }");
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_unchecked_overflow() {
    helper(
      unsafe { wrap_unchecked(|instant, _| instant.accept_unchecked(4).into()) },
      "1",
      0,
    );
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_unchecked_invalid_code_point() {
    helper(
      unsafe { wrap_unchecked(|instant, _| instant.accept_unchecked(1).into()) },
      "好",
      0,
    );
  }

  #[test]
  fn combinator_wrap() {
    let c = wrap(|instant, _| instant.accept(1));
    helper(c, "1", 1);

    // ensure the combinator is copyable and clone-able
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Wrap }");
  }

  #[test]
  fn combinator_wrap_fn() {
    fn action(instant: &Instant<&str>, _: Context<&mut (), &mut ()>) -> Option<Output<()>> {
      instant.accept(1)
    }
    let c = wrap(action);
    helper(c, "1", 1);

    // ensure the combinator is copyable and clone-able
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Wrap }");
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_overflow() {
    helper(
      wrap(|instant, _| unsafe { instant.accept_unchecked(4) }.into()),
      "1",
      0,
    );
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_invalid_code_point() {
    helper(
      wrap(|instant, _| unsafe { instant.accept_unchecked(1) }.into()),
      "好",
      0,
    );
  }

  #[test]
  fn combinator_bytes_wrap_unchecked() {
    let c = unsafe { bytes::wrap_unchecked(|instant, _| instant.accept(1)) };
    helper(c, b"1", 1);

    // ensure the combinator is copyable and clone-able
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: WrapUnchecked }");
  }

  #[test]
  #[should_panic]
  fn combinator_bytes_wrap_unchecked_overflow() {
    helper(
      unsafe { bytes::wrap_unchecked(|instant, _| instant.accept_unchecked(4).into()) },
      b"1",
      0,
    );
  }

  #[test]
  fn combinator_bytes_wrap() {
    let c = bytes::wrap(|instant, _| instant.accept(1));
    helper(c, b"1", 1);

    // ensure the combinator is copyable and clone-able
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Wrap }");
  }

  #[test]
  #[should_panic]
  fn combinator_bytes_wrap_overflow() {
    helper(
      bytes::wrap(|instant, _| unsafe { instant.accept_unchecked(4) }.into()),
      b"1",
      0,
    );
  }
}
