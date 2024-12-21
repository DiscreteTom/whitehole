//! The basic building block of a parser.
//!
//! Each action is a small piece of parsing logic that
//! digest some bytes from the input, optionally change the state of the parsing,
//! and yield a value.

mod input;
mod output;

pub use input::*;
pub use output::*;

/// The basic building block of a parser.
/// See the [module level documentation](crate::action) for more information.
pub trait Action {
  /// See [`Output::value`].
  type Value;
  /// See [`Input::state`].
  type State;
  /// See [`Input::heap`].
  type Heap;

  /// Return [`None`] to reject.
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Value>>;
}
