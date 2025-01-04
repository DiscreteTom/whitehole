use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
};
use core::{fmt, marker::PhantomData};

macro_rules! impl_wrap {
  ($name:ident, $assert:ident) => {
    impl<T, State, Heap> $name<T, State, Heap> {
      #[inline]
      const fn new(inner: T) -> Self {
        Self {
          inner,
          _phantom: PhantomData,
        }
      }
    }

    impl<T, State, Heap> fmt::Debug for $name<T, State, Heap> {
      #[inline]
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!($name)).finish()
      }
    }

    impl<T: Copy, State, Heap> Copy for $name<T, State, Heap> {}

    impl<T: Clone, State, Heap> Clone for $name<T, State, Heap> {
      #[inline]
      fn clone(&self) -> Self {
        Self {
          inner: self.inner.clone(),
          _phantom: PhantomData,
        }
      }
    }

    unsafe impl<Value, State, Heap, F: Fn(Input<&mut State, &mut Heap>) -> Option<Output<Value>>>
      Action for $name<F, State, Heap>
    {
      type Value = Value;
      type State = State;
      type Heap = Heap;

      #[inline]
      fn exec(
        &self,
        input: Input<&mut Self::State, &mut Self::Heap>,
      ) -> Option<Output<Self::Value>> {
        let input_rest = input.instant().rest();
        let output = (self.inner)(input);
        $assert!(output
          .as_ref()
          .map_or(true, |output| output.digested <= input_rest.len()
            && input_rest.is_char_boundary(output.digested)));
        output
      }
    }
  };
}

/// See [`wrap_unchecked`].
pub struct WrapUnchecked<F, State = (), Heap = ()> {
  inner: F,
  _phantom: PhantomData<(State, Heap)>,
}

impl_wrap!(WrapUnchecked, debug_assert);

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
) -> Combinator<WrapUnchecked<F, State, Heap>> {
  Combinator::new(WrapUnchecked::new(f))
}

/// See [`wrap`].
pub struct Wrap<F, State = (), Heap = ()> {
  inner: F,
  _phantom: PhantomData<(State, Heap)>,
}

impl_wrap!(Wrap, assert);

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
) -> Combinator<Wrap<F, State, Heap>> {
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
