//! Parse-related types and traits.

mod input;
mod output;

pub use input::*;
pub use output::*;

/// Provide the [`parse`](Parse::parse) method.
pub trait Parse {
  /// See [`Output::value`].
  type Value;
  /// See [`Input::state`].
  type State;
  /// See [`Input::heap`].
  type Heap;

  /// Return [`None`] to reject.
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Value>>;
}
