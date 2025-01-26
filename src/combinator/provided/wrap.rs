use crate::{
  action::{Action, Input, Output},
  combinator::{create_closure_combinator, Combinator},
  digest::Digest,
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
        F: Fn(Input<&$text, &mut State, &mut Heap>) -> Option<Output<Value>>,
      > Action<$text, State, Heap> for $name<F>
    {
      type Value = Value;

      #[inline]
      fn exec(&self, input: Input<&$text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
        let rest = input.instant().rest();
        let output = (self.inner)(input);
        $assert!(output
          .as_ref()
          .map_or(true, |output| rest.validate(output.digested)));
        output
      }
    }
  };
}

impl_wrap!(WrapUnchecked, debug_assert, str);
impl_wrap!(Wrap, assert, str);
impl_wrap!(WrapUnchecked, debug_assert, [u8]);
impl_wrap!(Wrap, assert, [u8]);

/// Wrap a closure to create a [`Combinator`].
///
/// For the bytes version, see [`bytes::wrap_unchecked`].
/// # Safety
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`].
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`wrap`].
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap_unchecked, Combinator};
/// # use whitehole::action::{Input, Output, Action};
/// # fn t() -> Combinator<impl Action> {
/// // eat the next character if it exists
/// unsafe { wrap_unchecked(|input| input.instant().rest().chars().next().and_then(|c| input.digest(c.len_utf8()))) }
/// # }
/// ```
#[inline]
pub const unsafe fn wrap_unchecked<
  F: Fn(Input<&str, &mut State, &mut Heap>) -> Option<Output<Value>>,
  Value,
  State,
  Heap,
>(
  f: F,
) -> Combinator<WrapUnchecked<F>> {
  Combinator::new(WrapUnchecked::new(f))
}

/// Wrap a closure to create a [`Combinator`].
///
/// For the bytes version, see [`bytes::wrap`].
/// # Panics
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`],
/// otherwise the combinator will panic when executed.
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap, Combinator};
/// # use whitehole::action::{Input, Output, Action};
/// # fn t() -> Combinator<impl Action> {
/// // eat the next character if it exists
/// wrap(|input| input.instant().rest().chars().next().and_then(|c| input.digest(c.len_utf8())))
/// # }
/// ```
#[inline]
pub const fn wrap<
  F: Fn(Input<&str, &mut State, &mut Heap>) -> Option<Output<Value>>,
  Value,
  State,
  Heap,
>(
  f: F,
) -> Combinator<Wrap<F>> {
  Combinator::new(Wrap::new(f))
}

pub mod bytes {
  use super::*;

  /// Wrap a closure to create a [`Combinator`] for bytes.
  ///
  /// For the string version, see [`wrap_unchecked`](super::wrap_unchecked).
  /// # Safety
  /// The returned [`Output`] should satisfy the requirement of [`Output::digested`].
  /// This will be checked using [`debug_assert!`].
  /// For the checked version, see [`wrap`].
  /// # Examples
  /// ```
  /// # use whitehole::combinator::{bytes::wrap_unchecked, Combinator};
  /// # use whitehole::action::{Input, Output, Action};
  /// # fn t() -> Combinator<impl Action> {
  /// // eat the next byte if it exists
  /// unsafe { wrap_unchecked(|input| input.digest(1)) }
  /// # }
  /// ```
  #[inline]
  pub const unsafe fn wrap_unchecked<
    F: Fn(Input<&[u8], &mut State, &mut Heap>) -> Option<Output<Value>>,
    Value,
    State,
    Heap,
  >(
    f: F,
  ) -> Combinator<WrapUnchecked<F>> {
    Combinator::new(WrapUnchecked::new(f))
  }

  /// Wrap a closure to create a [`Combinator`] for bytes.
  ///
  /// For the string version, see [`wrap`](super::wrap).
  /// # Panics
  /// The returned [`Output`] should satisfy the requirement of [`Output::digested`],
  /// otherwise the combinator will panic when executed.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::{bytes::wrap, Combinator};
  /// # use whitehole::action::{Input, Output, Action};
  /// # fn t() -> Combinator<impl Action> {
  /// // eat the next byte if it exists
  /// wrap(|input| input.digest(1))
  /// # }
  /// ```
  #[inline]
  pub const fn wrap<
    F: Fn(Input<&[u8], &mut State, &mut Heap>) -> Option<Output<Value>>,
    Value,
    State,
    Heap,
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

  #[test]
  fn combinator_wrap_unchecked() {
    let c = unsafe { wrap_unchecked(|input| input.digest(1)) };
    assert_eq!(
      c.exec(Input::new(Instant::new("1"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 1
      })
    );

    // ensure the combinator is copyable and clone-able
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: WrapUnchecked }");
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_unchecked_overflow() {
    unsafe { wrap_unchecked(|input| input.digest_unchecked(4).into()) }.exec(Input::new(
      Instant::new("1"),
      &mut (),
      &mut (),
    ));
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_unchecked_invalid_code_point() {
    unsafe { wrap_unchecked(|input| input.digest_unchecked(1).into()) }.exec(Input::new(
      Instant::new("好"),
      &mut (),
      &mut (),
    ));
  }

  #[test]
  fn combinator_wrap() {
    let c = wrap(|input| input.digest(1));
    assert_eq!(
      c.exec(Input::new(Instant::new("1"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 1
      })
    );

    // ensure the combinator is copyable and clone-able
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Wrap }");
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_overflow() {
    wrap(|input| unsafe { input.digest_unchecked(4) }.into()).exec(Input::new(
      Instant::new("1"),
      &mut (),
      &mut (),
    ));
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_invalid_code_point() {
    wrap(|input| unsafe { input.digest_unchecked(1) }.into()).exec(Input::new(
      Instant::new("好"),
      &mut (),
      &mut (),
    ));
  }

  #[test]
  fn combinator_bytes_wrap_unchecked() {
    let c = unsafe { bytes::wrap_unchecked(|input| input.digest(1)) };
    assert_eq!(
      c.exec(Input::new(Instant::new(b"1"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 1
      })
    );

    // ensure the combinator is copyable and clone-able
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: WrapUnchecked }");
  }

  #[test]
  #[should_panic]
  fn combinator_bytes_wrap_unchecked_overflow() {
    unsafe { bytes::wrap_unchecked(|input| input.digest_unchecked(4).into()) }.exec(Input::new(
      Instant::new(b"1"),
      &mut (),
      &mut (),
    ));
  }

  #[test]
  fn combinator_bytes_wrap() {
    let c = bytes::wrap(|input| input.digest(1));
    assert_eq!(
      c.exec(Input::new(Instant::new(b"1"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 1
      })
    );

    // ensure the combinator is copyable and clone-able
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Wrap }");
  }

  #[test]
  #[should_panic]
  fn combinator_bytes_wrap_overflow() {
    bytes::wrap(|input| unsafe { input.digest_unchecked(4) }.into()).exec(Input::new(
      Instant::new(b"1"),
      &mut (),
      &mut (),
    ));
  }
}
