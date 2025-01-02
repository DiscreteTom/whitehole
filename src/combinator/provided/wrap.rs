use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
  C,
};
use std::marker::PhantomData;

/// See [`wrap_unchecked`] and [`wrap`].
#[derive(Debug, Clone, Copy)]
struct WrapUnchecked<F, State = (), Heap = ()> {
  inner: F,
  _phantom: PhantomData<(State, Heap)>,
}

impl<T, State, Heap> WrapUnchecked<T, State, Heap> {
  #[inline]
  const fn new(inner: T) -> Self {
    Self {
      inner,
      _phantom: PhantomData,
    }
  }
}

unsafe impl<Value, State, Heap, F: Fn(Input<&mut State, &mut Heap>) -> Option<Output<Value>>> Action
  for WrapUnchecked<F, State, Heap>
{
  type Value = Value;
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec(&self, input: Input<&mut Self::State, &mut Self::Heap>) -> Option<Output<Self::Value>> {
    let input_rest = input.instant().rest();
    let output = (self.inner)(input);
    debug_assert!(output
      .as_ref()
      .map_or(true, |output| output.digested <= input_rest.len()
        && input_rest.is_char_boundary(output.digested)));
    output
  }
}

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
) -> C!(Value, State, Heap) {
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
) -> C!(Value, State, Heap) {
  unsafe {
    wrap_unchecked(move |input| {
      let rest = input.instant().rest();
      let output = f(input);
      assert!(output
        .as_ref()
        .map(|output| output.digested <= rest.len() && rest.is_char_boundary(output.digested))
        .unwrap_or(true));
      output
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::instant::Instant;

  #[test]
  fn combinator_wrap_unchecked() {
    assert_eq!(
      unsafe { wrap_unchecked(|input| input.digest(1)) }
        .exec(Input::new(Instant::new("1"), &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 1
      })
    );
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
    assert_eq!(
      wrap(|input| input.digest(1)).exec(Input::new(Instant::new("1"), &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 1
      })
    );
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
