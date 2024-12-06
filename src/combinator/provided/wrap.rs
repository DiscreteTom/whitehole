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
    Kind,
    State,
    Heap,
    F: for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Kind>>,
  > Parse for Wrap<F, State, Heap>
{
  type Kind = Kind;
  type State = State;
  type Heap = Heap;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    (self.inner)(input)
  }
}

/// Wrap a closure to create a [`Combinator`].
#[inline]
pub fn wrap<
  F: for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Kind>>,
  Kind,
  State,
  Heap,
>(
  parse: F,
) -> Combinator!(Kind, State, Heap) {
  Combinator::new(Wrap::new(parse))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn combinator_wrap() {
    assert_eq!(
      wrap(|input| Some(Output {
        kind: (),
        rest: &input.rest()[1..]
      }))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "23"
      })
    );
  }
}
