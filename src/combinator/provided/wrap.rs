use crate::{
  action::{Action, Input, Output},
  combinator::{create_closure_combinator, Combinator},
};

create_closure_combinator!(WrapUnchecked, "See [`wrap_unchecked`].");
create_closure_combinator!(Wrap, "See [`wrap`].");

macro_rules! impl_wrap {
  ($name:ident, $assert:ident) => {
    unsafe impl<Value, State, Heap, F: Fn(Input<&mut State, &mut Heap>) -> Option<Output<Value>>>
      Action<State, Heap> for $name<F>
    {
      type Value = Value;

      #[inline]
      fn exec(&self, mut input: Input<&mut State, &mut Heap>) -> Option<Output<Self::Value>> {
        let output = (self.inner)(input.reborrow());
        $assert!(output
          .as_ref()
          .map_or(true, |output| input.validate(output.digested)));
        output
      }
    }
  };
}

impl_wrap!(WrapUnchecked, debug_assert);
impl_wrap!(Wrap, assert);

/// Wrap a closure to create a [`Combinator`].
/// # Safety
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`].
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`wrap`].
/// # Examples
/// ```
/// # use whitehole::C;
/// # use whitehole::combinator::wrap_unchecked;
/// # use whitehole::action::{Input, Output};
/// # fn t() -> C!() {
/// // eat the next character
/// unsafe { wrap_unchecked(|input| input.digest(input.next().len_utf8())) }
/// # }
/// ```
#[inline]
pub const unsafe fn wrap_unchecked<
  F: Fn(Input<&mut State, &mut Heap>) -> Option<Output<Value>>,
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
/// # use whitehole::C;
/// # use whitehole::combinator::wrap;
/// # use whitehole::action::{Input, Output};
/// # fn t() -> C!() {
/// // eat the next character
/// wrap(|input| input.digest(input.next().len_utf8()))
/// # }
/// ```
#[inline]
pub const fn wrap<
  F: Fn(Input<&mut State, &mut Heap>) -> Option<Output<Value>>,
  Value,
  State,
  Heap,
>(
  f: F,
) -> Combinator<Wrap<F>> {
  Combinator::new(Wrap::new(f))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::instant::Instant;

  #[test]
  fn combinator_wrap_unchecked() {
    let c = unsafe { wrap_unchecked(|input| input.digest(1)) };
    assert_eq!(
      c.exec(Input::new(Instant::new("1"), &mut (), &mut ()).unwrap()),
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
    unsafe { wrap_unchecked(|input| input.digest_unchecked(4).into()) }
      .exec(Input::new(Instant::new("1"), &mut (), &mut ()).unwrap());
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_unchecked_invalid_code_point() {
    unsafe { wrap_unchecked(|input| input.digest_unchecked(1).into()) }
      .exec(Input::new(Instant::new("好"), &mut (), &mut ()).unwrap());
  }

  #[test]
  fn combinator_wrap() {
    let c = wrap(|input| input.digest(1));
    assert_eq!(
      c.exec(Input::new(Instant::new("1"), &mut (), &mut ()).unwrap()),
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
    wrap(|input| unsafe { input.digest_unchecked(4) }.into())
      .exec(Input::new(Instant::new("1"), &mut (), &mut ()).unwrap());
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_invalid_code_point() {
    wrap(|input| unsafe { input.digest_unchecked(1) }.into())
      .exec(Input::new(Instant::new("好"), &mut (), &mut ()).unwrap());
  }
}
