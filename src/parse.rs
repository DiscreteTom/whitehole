// mod closure;
mod input;
mod output;

// pub(crate) use closure::*;
pub use input::*;
pub use output::*;

/// Provide the [`parse`](Parse::parse) method.
pub trait Parse<Kind, State, Heap> {
  /// Return [`None`] if the combinator is rejected.
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Kind>>;
}

impl<
    Kind,
    State,
    Heap,
    T: for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Kind>>,
  > Parse<Kind, State, Heap> for T
{
  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Kind>> {
    self(input)
  }
}
