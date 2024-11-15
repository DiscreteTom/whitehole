use super::{Input, Output, Parse};
use std::marker::PhantomData;

/// Implement [`Parse`] for plain closures.
/// Currently this struct is required to constrain generic params.
/// Maybe removed in the future, thus private.
/// TODO: remove this.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Closure<Kind, State, Heap, T> {
  closure: T,
  _phantom: PhantomData<(Kind, State, Heap)>,
}

impl<Kind, State, Heap, T> Closure<Kind, State, Heap, T> {
  #[inline]
  pub(crate) fn new(closure: T) -> Self {
    Self {
      closure,
      _phantom: PhantomData,
    }
  }
}

impl<
    Kind,
    State,
    Heap,
    T: for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Kind>>,
  > Parse for Closure<Kind, State, Heap, T>
{
  type Kind = Kind;
  type State = State;
  type Heap = Heap;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    (self.closure)(input)
  }
}

// TODO: tests
