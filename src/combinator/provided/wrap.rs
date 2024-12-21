use crate::{
  combinator::Combinator,
  parse::{Input, Output, Parse},
  Combinator,
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

impl<
    Value,
    State,
    Heap,
    F: for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Value>>,
  > Parse for Wrap<F, State, Heap>
{
  type Value = Value;
  type State = State;
  type Heap = Heap;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Value>> {
    (self.inner)(input)
  }
}

/// Wrap a closure to create a [`Combinator`].
#[inline]
pub const fn wrap<
  F: for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Value>>,
  Value,
  State,
  Heap,
>(
  parse: F,
) -> Combinator!(Value, State, Heap) {
  Combinator::new(Wrap::new(parse))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn combinator_wrap() {
    assert_eq!(
      wrap(|input| Some(Output {
        value: (),
        rest: &input.rest()[1..]
      }))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "23"
      })
    );
  }
}
