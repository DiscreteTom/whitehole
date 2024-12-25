//! The basic building block of a parser.
//!
//! Each [`Action`] is a small piece of parsing logic that
//! digest some bytes from the input, optionally change the state of the parsing,
//! and yield a value.
//!
//! For most cases, you don't need to use [`Action`] directly.
//! See [`Combinator`](crate::combinator::Combinator) and
//! [`Parser`](crate::parser::Parser) for more high-level APIs.
//!
//! # Stateless
//!
//! [`Action`]s are stateless and immutable,
//! but they can access external states to change their behavior.
//! See [`Input::state`] and [`Input::heap`] for more information.
//!
//! States are centrally managed by the parser,
//! so it's easy to realize peeking and backtracking.
//!
//! # Consume the [`Input`]
//!
//! If not consuming the `Input`:
//! - With `&Input`: [`Input::state`] and [`Input::heap`] can't be mutated.
//! - With `&mut Input`: [`Action`]s may [`std::mem::swap`] the `Input` to break the outer state.
//!
//! So we decide to consume the `Input` in [`Action::exec`].
//! If you need to use `Input` for multiple times, see [`Input::reborrow`].

mod input;
mod output;

pub use input::*;
pub use output::*;

/// The basic building block of a parser.
/// See the [module level documentation](crate::action) for more information.
/// # Safety
/// The [`Output`] of [`Action::exec`] should satisfy the requirement of [`Output::digested`].
pub unsafe trait Action {
  /// See [`Output::value`].
  type Value;
  /// See [`Input::state`].
  type State;
  /// See [`Input::heap`].
  type Heap;

  /// Try to digest some bytes from the input, optionally change the state of the parsing,
  /// and yield a value.
  /// Return [`None`] to reject.
  fn exec<'text>(
    &self,
    input: Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>>;
}
