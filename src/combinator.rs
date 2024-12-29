//! [`Combinator`] is a wrapper around [`Action`] to provide decorators and operator overloads.
//! # Basic Usage
//! ## Provided Combinators
//! To get started, you can use the provided combinators like [`eat`],
//! which will eat the provided pattern from the rest of the input text:
//! ```
//! # use whitehole::{combinator::eat, C};
//! # fn t(_: C!()) {}
//! # t(
//! eat("true")
//! # );
//! ```
//! To save the memory of your brain, we have very limited number of provided combinators.
//! Here is the full list:
//! - [`eat`]: eat a pattern.
//! - [`eater`]: eat by a custom function.
//! - [`next`]: eat the next character by a predicate.
//! - [`till`]: eat until a pattern, inclusive.
//! - [`wrap`]: wrap a closure as a combinator.
//!
//! Tips: Some of them may have faster `unsafe` variants named with suffix `_unchecked`.
//! ## Composition
//! Use `+` and `|` to compose multiple combinators
//! for more complex tasks:
//! ```
//! # use whitehole::{combinator::eat, C};
//! # fn t(_: C!()) {}
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
//! # use whitehole::{combinator::eat, C};
//! # fn t(_: C!()) {}
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
  action: T,
}

/// Simplify the [`Combinator`] struct's signature.
///
/// - `C!()` => `Combinator<impl Action<Value = (), State = (), Heap = ()>>`.
/// - `C!(MyValue)` => `Combinator<impl Action<Value = MyValue, State = (), Heap = ()>>`.
/// - `C!(MyValue, MyState)` => `Combinator<impl Action<Value = MyValue, State = MyState, Heap = ()>>`.
/// - `C!(MyValue, MyState, MyHeap)` => `Combinator<impl Action<Value = MyValue, State = MyState, Heap = MyHeap>>`.
/// - `C!(@T)` => `Combinator<impl Action<Value = T::Value, State = T::State, Heap = T::Heap>>`.
/// - `C!(MyValue, @T)` => `Combinator<impl Action<Value = MyValue, State = T::State, Heap = T::Heap>>`.
#[macro_export]
macro_rules! C {
  () => {
    $crate::combinator::Combinator<impl $crate::action::Action<Value = (), State = (), Heap = ()>>
  };
  ($value:ty) => {
    $crate::combinator::Combinator<impl $crate::action::Action<Value = $value, State = (), Heap = ()>>
  };
  ($value:ty, $state:ty) => {
    $crate::combinator::Combinator<impl $crate::action::Action<Value = $value, State = $state, Heap = ()>>
  };
  ($value:ty, $state:ty, $heap:ty) => {
    $crate::combinator::Combinator<impl $crate::action::Action<Value = $value, State = $state, Heap = $heap>>
  };
  (@$from:ident) => {
    $crate::combinator::Combinator<impl $crate::action::Action<Value = $from::Value, State = $from::State, Heap = $from::Heap>>
  };
  ($value:ty, @$from:ident) => {
    $crate::combinator::Combinator<impl $crate::action::Action<Value = $value, State = $from::State, Heap = $from::Heap>>
  };
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
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<T::Value>> {
    self.action.exec(input)
  }
}
