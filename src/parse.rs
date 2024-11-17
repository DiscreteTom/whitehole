//! Parse-related types and traits.

mod input;
mod output;

pub use input::*;
pub use output::*;

/// Provide the [`parse`](Parse::parse) method.
pub trait Parse<State = (), Heap = ()> {
  /// See [`Output::kind`].
  type Kind;

  /// Return [`None`] to reject.
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

#[cfg(test)]
mod tests {
  use super::*;

  /// Ensure closures implement [`Parse`].
  fn _closure() {
    fn assert_parse(_: impl Parse<(), (), Kind = ()>) {}

    /// A helper function to cast a closure to a [`Parse`] implementation.
    fn cast<Kind, State, Heap>(
      f: impl for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Kind>>,
    ) -> impl Parse<State, Heap, Kind = Kind> {
      f
    }

    assert_parse(cast(|_| None));
    assert_parse(cast(|input| input.digest(0)));
  }
}
