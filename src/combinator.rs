//! [`Combinator`] is a wrapper around [`Action`] to provide decorators and operator overloads.
//! # Basic Usage
//! ## Provided Combinators
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
//! - [`next`]: eat the next character by a predicate.
//! - [`till`]: eat until a pattern, inclusive.
//! - [`wrap`]: wrap a closure as a combinator.
//!
//! Tips: Some of them may have faster `unsafe` variants named with suffix `_unchecked`.
//! ## Composition
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
//! ## Repetition
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
//! ## Decorator
//! [`Combinator`] provides a set of methods as decorators
//! to modify the behavior of the combinator.
//! ### Flow Control
//! - [`Combinator::optional`] to make a combinator optional.
//! - [`Combinator::boundary`] to require a word boundary after the action is accepted.
//! - [`Combinator::when`] to conditionally execute the combinator.
//! - [`Combinator::prevent`] to conditionally reject the combinator before it is executed.
//! - [`Combinator::reject`] to conditionally reject the combinator after it is executed.
//! ### Value Transformation
//! You can set [`Output::value`] to distinguish different output types
//! or carrying additional data.
//!
//! Related decorators:
//! - [`Combinator::map`] to convert the value to a new value.
//! - [`Combinator::tuple`] to wrap the value in an one-element tuple.
//! - [`Combinator::pop`] to unwrap the value from the one-element tuple.
//! - [`Combinator::bind`] to set the value to a provided value.
//! - [`Combinator::bind_default`] to set the value to the default value.
//! - [`Combinator::select`] to calculate the value with a closure.
//! - [`Combinator::range`] to wrap the value in a [`WithRange`](crate::range::WithRange) struct.
//! ### State Manipulation
//! [`Combinator`]s are stateless, but you can access external states
//! via [`Input::state`] to realize stateful parsing.
//!
//! Related decorators:
//! - [`Combinator::prepare`] to modify states before being executed.
//! - [`Combinator::then`] to modify states after being accepted.
//! - [`Combinator::catch`] to modify states after being rejected.
//! - [`Combinator::finally`] to modify states after being executed.

mod decorator;
mod provided;

pub mod ops;

pub use decorator::*;
pub use provided::*;

use crate::action::{Action, Input, Output};

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

unsafe impl<Text: ?Sized, State, Heap, T: Action<Text, State, Heap>> Action<Text, State, Heap>
  for Combinator<T>
{
  type Value = T::Value;

  #[inline]
  fn exec(&self, input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<T::Value>> {
    self.action.exec(input)
  }
}

macro_rules! create_combinator {
  ($name:ident, $usage:literal, ($($derives:ident),*)) => {
    #[doc = $usage]
    #[derive(Copy, Clone, $($derives),*)]
    pub struct $name<T> {
      inner: T,
    }

    impl<T> $name<T> {
      #[inline]
      const fn new(inner: T) -> Self {
        Self { inner }
      }
    }
  };
}
// https://github.com/rust-lang/rust-clippy/issues/12808
#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use create_combinator;

macro_rules! create_value_combinator {
  ($name:ident, $usage:literal) => {
    $crate::combinator::create_combinator!($name, $usage, (Debug));
  };
}
// https://github.com/rust-lang/rust-clippy/issues/12808
#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use create_value_combinator;

macro_rules! create_closure_combinator {
  ($name:ident, $usage:literal) => {
    $crate::combinator::create_combinator!($name, $usage, ());

    impl<T> core::fmt::Debug for $name<T> {
      #[inline]
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct(stringify!($name)).finish()
      }
    }
  };
}
// https://github.com/rust-lang/rust-clippy/issues/12808
#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use create_closure_combinator;
