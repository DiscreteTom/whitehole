mod input;
mod output;

pub use input::*;
pub use output::*;

/// Provide the [`parse`](Parse::parse) method.
pub trait Parse<State, Heap> {
  /// See [`Output::kind`].
  type Kind;

  /// Return [`None`] if the combinator is rejected.
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>>;
}

impl<
    Kind,
    State,
    Heap,
    T: for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Kind>>,
  > Parse<State, Heap> for T
{
  type Kind = Kind;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Kind>> {
    self(input)
  }
}
