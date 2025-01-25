use crate::{
  action::{Action, Input, Output},
  combinator::{create_closure_combinator, Combinator},
};

create_closure_combinator!(WrapUnchecked, "See [`wrap_unchecked`].");
create_closure_combinator!(Wrap, "See [`wrap`].");
create_closure_combinator!(WrapBytesUnchecked, "See [`wrap_bytes_unchecked`].");
create_closure_combinator!(WrapBytes, "See [`wrap_bytes`].");

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
      fn exec(
        &self,
        mut input: Input<&$text, &mut State, &mut Heap>,
      ) -> Option<Output<Self::Value>> {
        let output = (self.inner)(input.reborrow());
        $assert!(output
          .as_ref()
          .map_or(true, |output| input.validate(output.digested)));
        output
      }
    }
  };
}

impl_wrap!(WrapUnchecked, debug_assert, str);
impl_wrap!(Wrap, assert, str);
impl_wrap!(WrapBytesUnchecked, debug_assert, [u8]);
impl_wrap!(WrapBytes, assert, [u8]);

/// Wrap a closure to create a [`Combinator`].
/// # Safety
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`].
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`wrap`].
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap_unchecked, Combinator};
/// # use whitehole::action::{Input, Output, Action};
/// # fn t() -> Combinator<impl Action> {
/// // eat the next character
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
/// # Panics
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`],
/// otherwise the combinator will panic when executed.
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap, Combinator};
/// # use whitehole::action::{Input, Output, Action};
/// # fn t() -> Combinator<impl Action> {
/// // eat the next character
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

/// Wrap a closure to create a [`Combinator`] for bytes.
/// # Safety
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`].
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`wrap`].
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap_bytes_unchecked, Combinator};
/// # use whitehole::action::{Input, Output, Action};
/// # fn t() -> Combinator<impl Action> {
/// // eat the next character
/// unsafe { wrap_bytes_unchecked(|input| input.instant().rest().get(0).and_then(|c| input.digest(1))) }
/// # }
/// ```
#[inline]
pub const unsafe fn wrap_bytes_unchecked<
  F: Fn(Input<&[u8], &mut State, &mut Heap>) -> Option<Output<Value>>,
  Value,
  State,
  Heap,
>(
  f: F,
) -> Combinator<WrapBytesUnchecked<F>> {
  Combinator::new(WrapBytesUnchecked::new(f))
}

/// Wrap a closure to create a [`Combinator`] for bytes.
/// # Panics
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`],
/// otherwise the combinator will panic when executed.
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap_bytes, Combinator};
/// # use whitehole::action::{Input, Output, Action};
/// # fn t() -> Combinator<impl Action> {
/// // eat the next character
/// wrap_bytes(|input| input.instant().rest().get(0).and_then(|c| input.digest(1)))
/// # }
/// ```
#[inline]
pub const fn wrap_bytes<
  F: Fn(Input<&[u8], &mut State, &mut Heap>) -> Option<Output<Value>>,
  Value,
  State,
  Heap,
>(
  f: F,
) -> Combinator<WrapBytes<F>> {
  Combinator::new(WrapBytes::new(f))
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
  fn combinator_wrap_bytes_unchecked() {
    let c = unsafe { wrap_bytes_unchecked(|input| input.digest(1)) };
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
    assert_eq!(
      format!("{:?}", c),
      "Combinator { action: WrapBytesUnchecked }"
    );
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_bytes_unchecked_overflow() {
    unsafe { wrap_bytes_unchecked(|input| input.digest_unchecked(4).into()) }.exec(Input::new(
      Instant::new(b"1"),
      &mut (),
      &mut (),
    ));
  }

  #[test]
  fn combinator_wrap_bytes() {
    let c = wrap_bytes(|input| input.digest(1));
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
    assert_eq!(format!("{:?}", c), "Combinator { action: WrapBytes }");
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_bytes_overflow() {
    wrap_bytes(|input| unsafe { input.digest_unchecked(4) }.into()).exec(Input::new(
      Instant::new(b"1"),
      &mut (),
      &mut (),
    ));
  }
}
