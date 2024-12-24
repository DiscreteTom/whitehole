use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
  C,
};
use std::marker::PhantomData;

/// See [`wrap`].
#[derive(Debug, Clone, Copy)]
struct Wrap<F, State = (), Heap = ()> {
  inner: F,
  _phantom: PhantomData<(State, Heap)>,
}

impl<T, State, Heap> Wrap<T, State, Heap> {
  #[inline]
  const fn new(inner: T) -> Self {
    Self {
      inner,
      _phantom: PhantomData,
    }
  }
}

unsafe impl<
    Value,
    State,
    Heap,
    F: for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<Value>>,
  > Action for Wrap<F, State, Heap>
{
  type Value = Value;
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let output = (self.inner)(input);
    debug_assert!(output
      .as_ref()
      .map(|output| output.digested <= input.rest().len()
        && input.rest().is_char_boundary(output.digested))
      .unwrap_or(true));
    output
  }
}

/// Wrap a closure to create a [`Combinator`].
/// # Safety
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`].
/// This will be checked using [`debug_assert!`].
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
pub const unsafe fn wrap<
  F: for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<Value>>,
  Value,
  State,
  Heap,
>(
  f: F,
) -> C!(Value, State, Heap) {
  Combinator::new(Wrap::new(f))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn combinator_wrap() {
    assert_eq!(
      unsafe { wrap(|input| input.digest(1)) }
        .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 1
      })
    );
  }
}
