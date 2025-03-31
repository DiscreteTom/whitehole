//! [`Combinator`] is a wrapper around [`Action`] to provide decorators and operator overloads.
//! # Provided Combinators
//! To get started, you can use the provided combinators like [`eat`],
//! which will eat the provided pattern from the rest of the input text:
//! ```
//! # use whitehole::{combinator::{eat, Combinator}, action::Action};
//! # fn t(_: Combinator<impl Action>) {}
//! # t(
//! eat("true")
//! # );
//! ```
//! To save the memory of your brain, we have very limited number of provided combinators.
//! Here is the full list:
//! - [`eat`]: eat a pattern.
//! - [`take`]: take the next `n` chars or bytes.
//! - [`next`]: eat the next character by a predicate.
//! - [`till`]: eat until a pattern, inclusive.
//! - [`wrap`]: wrap a closure as a combinator.
//! - [`recur`]: create a recursive combinator.
//!
//! Tips:
//! - Some of them may have faster `unsafe` variants named with suffix `_unchecked`.
//! - Some of them can be used in both string and bytes context.
//!   Some may have specific variants for bytes under the [`bytes`] module.
//! # Composition
//! Use `+` and `|` to compose multiple combinators
//! for more complex tasks:
//! ```
//! # use whitehole::{combinator::{eat, Combinator}, action::Action};
//! # fn t(_: Combinator<impl Action>) {}
//! // match "true" then match "false"
//! # t(
//! eat("true") + eat("false")
//! # );
//! // match "true" or "false"
//! # t(
//! eat("true") | eat("false")
//! # );
//! ```
//! See [`ops::add`] and [`ops::bitor`] for more information.
//! # Repetition
//! Use `*` to repeat a combinator:
//! ```
//! # use whitehole::{combinator::{eat, Combinator}, action::Action};
//! # fn t(_: Combinator<impl Action>) {}
//! // repeat the combinator for 2 times
//! # t(
//! eat("true") * 2
//! # );
//! // equals to
//! # t(
//! eat("true") + "true"
//! # );
//!
//! // repeat the combinator with a range, greedy
//! # t(
//! eat("true") * (1..=3)
//! # );
//! // similar to but faster than
//! # t(
//! (eat("true") + "true" + "true") | (eat("true") + "true") | eat("true")
//! # );
//! ```
//! See [`ops::mul`] for more information.
//! # Lookahead
//! Use `!` for negative lookahead:
//! ```
//! # use whitehole::{combinator::{eat, Combinator}, action::Action};
//! # fn t(_: Combinator<impl Action>) {}
//! // reject if the next char is 'a', otherwise accept with 0 digested
//! // (negative lookahead)
//! # t(
//! !eat('a')
//! # );
//! // apply twice to realize positive lookahead
//! # t(
//! !!eat('a')
//! # );
//! ```
//! See [`ops::not`] for more information.
//! # Decorator
//! [`Combinator`] provides a set of methods as decorators
//! to modify the behavior of the combinator.
//! ## Debug
//! - [`Combinator::log`] to print debug information.
//! ## Flow Control
//! - [`Combinator::optional`] to make a combinator optional.
//! - [`Combinator::boundary`] to require a word boundary after the action is accepted.
//! - [`Combinator::when`] to conditionally execute the combinator.
//! - [`Combinator::prevent`] to conditionally reject the combinator before it is executed.
//! - [`Combinator::reject`] to conditionally reject the combinator after it is executed.
//! ## Value Transformation
//! You can set [`Output::value`] to distinguish different output types
//! or carrying additional data.
//!
//! Related decorators:
//! - [`Combinator::map`] to convert the value to a new value.
//! - [`Combinator::tuple`] to wrap the value in an one-element tuple.
//! - [`Combinator::pop`] to unwrap the value from the one-element tuple.
//! - [`Combinator::bind`] to set the value to a provided clone-able value.
//! - [`Combinator::bind_with`] to set the value with a provided factory.
//! - [`Combinator::select`] to calculate the value with a closure.
//! - [`Combinator::range`] to wrap the value in a [`WithRange`](crate::range::WithRange) struct.
//! ## State Manipulation
//! [`Combinator`]s are stateless, but you can access external states
//! via [`Context::state`] to realize stateful parsing.
//!
//! Related decorators:
//! - [`Combinator::prepare`] to modify states before being executed.
//! - [`Combinator::then`] to modify states after being accepted.
//! - [`Combinator::catch`] to modify states after being rejected.
//! - [`Combinator::finally`] to modify states after being executed.

mod decorator;
mod provided;

pub mod ops;

pub use crate::contextual;
pub use decorator::*;
pub use provided::*;

use crate::{
  action::{Action, Input, Output},
  instant::Instant,
};

/// Wrap an [`Action`] to provide decorators and operator overloads.
///
/// See the [module-level documentation](self) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Combinator<T> {
  pub action: T,
}

impl<T> Combinator<T> {
  /// Create a new instance.
  ///
  /// For most cases you don't need to use this method directly.
  /// See the [module-level documentation](self) for more information.
  #[inline]
  pub const fn new(action: T) -> Self {
    Self { action }
  }
}

unsafe impl<T: Action> Action for Combinator<T> {
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut T::State, &mut T::Heap>,
  ) -> Option<Output<T::Value>> {
    self.action.exec(input)
  }
}
