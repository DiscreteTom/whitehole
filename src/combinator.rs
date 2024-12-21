//! The building block of a lexer or a parser.
//! Provide decorators and operator overloads for [`Parse`] implementors.
//! # Basic Usage
//! ## Provided Combinators
//! To get started, you can use the provided combinators like [`eat`],
//! which will eat the provided pattern from the rest of the input text:
//! ```
//! # use whitehole::{combinator::eat, Combinator};
//! # fn t(_: Combinator!()) {}
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
//! # fn t(_: Combinator!()) {}
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
//! # fn t(_: Combinator!()) {}
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
//! # fn t(_: Combinator!()) {}
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
//! ## Stateful
//! [`Combinator`]s are stateless, but you can access external states
//! via [`Input::state`] to realize stateful parsing.
//!
//! Related decorators:
//! - [`Combinator::prepare`]
//! - [`Combinator::then`]
//! - [`Combinator::rollback`]
//! - [`Combinator::prevent`]
//! - [`Combinator::reject`]

mod decorator;
mod provided;

pub mod ops;

pub use decorator::*;
pub use provided::*;

use crate::parse::{Input, Output, Parse};

/// Wrap a [`Parse`] implementor to provide decorators and operator overloads.
///
/// See the [module-level documentation](self) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Combinator<T> {
  parser: T,
}

/// Simplify the [`Combinator`] struct's signature.
///
/// - `Combinator!()` => `Combinator<impl Parse<Value = (), State = (), Heap = ()>>`.
/// - `Combinator!(MyValue)` => `Combinator<impl Parse<Value = MyValue, State = (), Heap = ()>>`.
/// - `Combinator!(MyValue, MyState)` => `Combinator<impl Parse<Value = MyValue, State = MyState, Heap = ()>>`.
/// - `Combinator!(MyValue, MyState, MyHeap)` => `Combinator<impl Parse<Value = MyValue, State = MyState, Heap = MyHeap>>`.
/// - `Combinator!(@T)` => `Combinator<impl Parse<Value = T::Value, State = T::State, Heap = T::Heap>>`.
/// - `Combinator!(MyValue, @T)` => `Combinator<impl Parse<Value = MyValue, State = T::State, Heap = T::Heap>>`.
#[macro_export]
macro_rules! Combinator {
  () => {
    $crate::combinator::Combinator<impl $crate::parse::Parse<Value = (), State = (), Heap = ()>>
  };
  ($value:ty) => {
    $crate::combinator::Combinator<impl $crate::parse::Parse<Value = $value, State = (), Heap = ()>>
  };
  ($value:ty, $state:ty) => {
    $crate::combinator::Combinator<impl $crate::parse::Parse<Value = $value, State = $state, Heap = ()>>
  };
  ($value:ty, $state:ty, $heap:ty) => {
    $crate::combinator::Combinator<impl $crate::parse::Parse<Value = $value, State = $state, Heap = $heap>>
  };
  (@$from:ident) => {
    $crate::combinator::Combinator<impl $crate::parse::Parse<Value = $from::Value, State = $from::State, Heap = $from::Heap>>
  };
  ($value:ty, @$from:ident) => {
    $crate::combinator::Combinator<impl $crate::parse::Parse<Value = $value, State = $from::State, Heap = $from::Heap>>
  };
}

impl<T> Combinator<T> {
  /// Create a new instance by wrapping a [`Parse`] implementor.
  #[inline]
  pub const fn new(parser: T) -> Self {
    Self { parser }
  }
}

impl<T: Parse> Parse for Combinator<T> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, T::Value>> {
    self.parser.parse(input)
  }
}
