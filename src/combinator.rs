//! The building block of a lexer or a parser.
//! Provide decorators and operator overloads for [`Action`]s.
//! # Basic Usage
//! ## Provided Combinators
//! To get started, you can use the provided combinators like [`eat`],
//! which will eat the provided pattern from the rest of the input text:
//! ```
//! # use whitehole::{combinator::eat, Combinator};
//! # fn t(_: C!()) {}
//! # t(
//! eat("true")
//! # );
//! ```
//! To save the memory of your brain, we have very limited number of provided combinators.
//! Here is the full list:
//! - General combinators for most cases.
//!   - [`eat`]: eat a pattern.
//!   - [`next`]: eat the next character by a predicate.
//!   - [`till`]: eat until a pattern.
//! - Advanced combinators if you want better performance or more customization.
//!   - [`eat_unchecked`]: eat `n` bytes without checking.
//!   - [`eater`]: eat by a custom function.
//!   - [`eater_unchecked`]: eat by a custom function without checking.
//!   - [`wrap`]: wrap a closure as a combinator.
//! ## Composition
//! Use `+` and `|` to compose multiple combinators
//! for more complex tasks:
//! ```
//! # use whitehole::{combinator::eat, Combinator};
//! # fn t(_: C!()) {}
//! // match "true" then match "false"
//! # t(
//! eat("true") + eat("false")
//! # );
//! // match "true" or "false"
//! # t(
//! eat("true") | eat("false")
//! # );
//!
//! // you can use a String, a &str, a char or an usize as a shortcut for `eat`
//! // at the right-hand side of `+` or `|`
//! # t(
//! eat("true") + "false"
//! # );
//! # t(
//! eat("true") | "false"
//! # );
//! # t(
//! eat("true") + ';'
//! # );
//! # t(
//! eat("true") | ';'
//! # );
//! # t(
//! eat("true") + 1
//! # );
//! # t(
//! eat("true") | 1
//! # );
//! ```
//! ## Repeat
//! Use `*` to repeat a combinator:
//! ```
//! # use whitehole::{combinator::eat, Combinator};
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
//! (eat("true") + "true" + "true") | (eat("true") + "true") |  eat("true")
//! # );
//!
//! // repeat for 0 or more times
//! # t(
//! eat("true") * (..)
//! # );
//! # t(
//! eat("true") * (..=3)
//! # );
//!
//! // repeating for 0 times will always accept with 0 bytes digested
//! # t(
//! eat("true") * 0
//! # );
//! # t(
//! eat("true") * (..1)
//! # );
//! # t(
//! eat("true") * (..=0)
//! # );
//!
//! // repeat with another combinator as the separator
//! # t(
//! eat("true") * (1.., eat(','))
//! # );
//! // you can use a String, a &str or a char as the separator
//! # t(
//! eat("true") * (1.., ',')
//! # );
//! # t(
//! eat("true") * (1.., ", ")
//! # );
//! # t(
//! eat("true") * (1.., ", ".to_string())
//! # );
//! ```
//! ## Decorator
//! [`Combinator`] provides a set of methods as decorators
//! to modify the behavior of the combinator.
//! For now let's see 2 of them:
//! ```
//! # use whitehole::{combinator::eat, Combinator};
//! # fn t(_: C!()) {}
//! // make the combinator optional
//! # t(
//! eat("true").optional()
//! # );
//! // require a word boundary after the combinator is accepted
//! # t(
//! eat("true").boundary()
//! # );
//! ```
//! ## Value
//! You can set [`Output::value`] to distinguish different output types
//! or carrying additional data.
//!
//! Related decorators:
//! - [`Combinator::bind`]
//! - [`Combinator::bind_default`]
//! - [`Combinator::select`]
//! - [`Combinator::map`]
//! - [`Combinator::tuple`]
//! - [`Combinator::range`]
//! ## Stateful
//! [`Combinator`]s are stateless, but you can access external states
//! via [`Input::state`] to realize stateful parsing.
//!
//! Related decorators:
//! - [`Combinator::prepare`]
//! - [`Combinator::then`]
//! - [`Combinator::catch`]
//! - [`Combinator::prevent`]
//! - [`Combinator::reject`]

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

impl<T: Action> Action for Combinator<T> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, T::Value>> {
    self.action.exec(input)
  }
}
