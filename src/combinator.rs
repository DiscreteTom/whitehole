//! The building block of a lexer or a parser.
//! # Basic Usage
//! ## Provided Combinators
//! To get started, you can use the provided combinators like [`eat`],
//! which will eat the provided pattern from the rest of the input text:
//! ```
//! # use whitehole::combinator::{Combinator, eat};
//! let _: Combinator<_> = eat("true");
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
//! # use whitehole::combinator::{Combinator, eat};
//! // match "true" then match "false"
//! let _: Combinator<_> = eat("true") + eat("false");
//!
//! // match "true" or "false"
//! let _: Combinator<_> = eat("true") | eat("false");
//!
//! // you can use a String, a &str, a char or an usize as a shortcut for `eat`
//! // at the right-hand side of `+` or `|`
//! let _: Combinator<_> = eat("true") + "false";
//! let _: Combinator<_> = eat("true") | "false";
//! let _: Combinator<_> = eat("true") + ';';
//! let _: Combinator<_> = eat("true") | ';';
//! let _: Combinator<_> = eat("true") + 1;
//! let _: Combinator<_> = eat("true") | 1;
//! ```
//! ## Repeat
//! Use `*` to repeat a combinator:
//! ```
//! # use whitehole::combinator::{Combinator, eat};
//! // repeat the combinator for 2 times
//! let _: Combinator<_> = eat("true") * 2;
//! // equals to
//! let _: Combinator<_> = eat("true") + "true";
//!
//! // repeat the combinator with a range, greedy
//! let _: Combinator<_> = eat("true") * (1..=3);
//! // similar to but faster than
//! let _: Combinator<_> =
//!     (eat("true") + "true" + "true")
//!   | (eat("true") + "true")
//!   |  eat("true");
//!
//! // repeat for 0 or more times
//! let _: Combinator<_> = eat("true") * (..);
//! let _: Combinator<_> = eat("true") * (..=3);
//!
//! // repeating for 0 times will always accept with 0 bytes digested
//! let _: Combinator<_> = eat("true") * 0;
//! let _: Combinator<_> = eat("true") * (..1);
//! let _: Combinator<_> = eat("true") * (..=0);
//!
//! // repeat with another combinator as the separator
//! let _: Combinator<_> = eat("true") * (1.., eat(','));
//! // you can use a String, a &str or a char as the separator
//! let _: Combinator<_> = eat("true") * (1.., ',');
//! let _: Combinator<_> = eat("true") * (1.., ", ");
//! let _: Combinator<_> = eat("true") * (1.., ", ".to_string());
//! ```
//! ## Decorator
//! [`Combinator`] provides a set of methods as decorators
//! to modify the behavior of the combinator.
//! For now let's see 2 of them:
//! ```
//! # use whitehole::combinator::{Combinator, eat};
//! // make the combinator optional
//! let _: Combinator<_> = eat("true").optional();
//! // require a word boundary after the combinator is accepted
//! let _: Combinator<_> = eat("true").boundary();
//! ```
//! ## Kind
//! You can set [`Output::kind`] to distinguish different output types
//! or carrying additional data.
//!
//! Related decorators:
//! - [`Combinator::bind`]
//! - [`Combinator::bind_default`]
//! - [`Combinator::select`]
//! - [`Combinator::map`]
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

mod builder;
mod decorator;
mod provided;

pub mod ops;

pub use builder::*;
pub use decorator::*;
pub use provided::*;

use crate::parse::{Input, Output, Parse};
use std::marker::PhantomData;

/// Wrap a [`Parse`] implementor to provide decorators and operator overloads.
///
/// See the [module-level documentation](self) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Combinator<T, State = (), Heap = ()> {
  parser: T,
  _phantom: PhantomData<(State, Heap)>,
}

/// Simplify the [`Combinator`] struct's signature.
///
/// - `Combinator!()` will be expanded to `Combinator<impl Parse<Kind = ()>>`.
/// - `Combinator!(MyKind)` will be expanded to `Combinator<impl Parse<Kind = MyKind>>`.
/// - `Combinator!(MyKind, State)` will be expanded to `Combinator<impl Parse<State, Kind = MyKind>, State>`.
/// - `Combinator!(MyKind, State, Heap)` will be expanded to `Combinator<impl Parse<State, Heap, Kind = MyKind>, State, Heap>`.
/// - `Combinator!(_, State, Heap)` will be expanded to `Combinator<impl Parse<State, Heap>, State, Heap>`.
#[macro_export]
macro_rules! Combinator {
  () => {
    $crate::combinator::Combinator<impl $crate::parse::Parse<Kind = ()>>
  };
  ($kind:ty) => {
    $crate::combinator::Combinator<impl $crate::parse::Parse<Kind = $kind>>
  };
  ($kind:ty, $state:ty) => {
    $crate::combinator::Combinator<impl $crate::parse::Parse<$state, Kind = $kind>, $state>
  };
  ($kind:ty, $state:ty, $heap:ty) => {
    $crate::combinator::Combinator<impl $crate::parse::Parse<$state, $heap, Kind = $kind>, $state, $heap>
  };
  (_, $state:ty, $heap:ty) => {
    $crate::combinator::Combinator<impl $crate::parse::Parse<$state, $heap>, $state, $heap>
  };
}

impl<T, State, Heap> Combinator<T, State, Heap> {
  /// Create a new instance by wrapping a [`Parse`] implementor.
  ///
  /// To wrap a closure, use [`wrap`] instead.
  #[inline]
  pub fn new(parser: T) -> Self {
    Self {
      parser,
      _phantom: PhantomData,
    }
  }

  // TODO
  // /// Simplify generic params.
  // #[inline]
  // pub fn collapse(self) -> Combinator!(_, State, Heap)
  // where
  //   T: Parse<State, Heap>,
  // {
  //   self
  // }
}

/// Wrap a closure to create a [`Combinator`].
#[inline]
pub fn wrap<Kind, State, Heap>(
  parse: impl for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Kind>>,
) -> Combinator!(Kind, State, Heap) {
  Combinator::new(parse)
}

impl<T: Parse<State, Heap>, State, Heap> Parse<State, Heap> for Combinator<T, State, Heap> {
  type Kind = T::Kind;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, T::Kind>> {
    self.parser.parse(input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn combinator_parse() {
    assert_eq!(
      wrap(|input| Some(Output {
        kind: (),
        rest: &input.rest()[1..]
      }))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "23"
      })
    );
  }
}
