mod closure;
mod input;
mod output;

pub(crate) use closure::*;
pub use input::*;
pub use output::*;

/// Provide the [`parse`](Parse::parse) method.
pub trait Parse {
  /// See [`Output::kind`].
  type Kind;
  /// See [`Input::state`].
  type State;
  /// See [`Input::heap`].
  type Heap;

  /// Return [`None`] if the combinator is rejected.
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Kind>>;
}
