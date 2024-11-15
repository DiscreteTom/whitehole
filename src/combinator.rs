//! The building block of a lexer or a parser.
//! # Basic Usage
//! ## Provided Combinators
//! To get started, you can use the provided combinators like [`exact`],
//! which will match a string or a char exactly:
//! ```
//! # use whitehole::combinator::{Combinator, exact};
//! let _: Combinator<_> = exact("true");
//! ```
//! See functions in this module for more provided combinators.
//! ## Combine
//! You can use operators to combine multiple combinators
//! to digest more complex content:
//! ```
//! # use whitehole::combinator::{Combinator, exact};
//! // match "true" then match "false"
//! let _: Combinator<_> = exact("true") + exact("false");
//!
//! // match "true" or "false"
//! let _: Combinator<_> = exact("true") | exact("false");
//!
//! // you can use a string or a char as a shortcut for `exact`
//! let _: Combinator<_> = exact("true") + "false";
//! let _: Combinator<_> = exact("true") | "false";
//!
//! // you can use an usize number as a shortcut for `eat`
//! // which will eat the next n bytes
//! let _: Combinator<_> = exact("true") + 1;
//! let _: Combinator<_> = exact("true") | 1;
//! ```
//! ## Repeat
//! To repeat a combinator, just use the `*` operator:
//! ```
//! # use whitehole::combinator::{Combinator, exact};
//! // repeat the combinator 2 times
//! let _: Combinator<_> = exact("true") * 2;
//! // equals to
//! let _: Combinator<_> = exact("true") + "true";
//!
//! // repeat the combinator with a range, greedy
//! let _: Combinator<_> = exact("true") * (1..=3);
//! // similar to but faster than
//! let _: Combinator<_> =
//!     (exact("true") + "true" + "true")
//!   | (exact("true") + "true")
//!   |  exact("true");
//!
//! // allowing repeat for 0 or more times
//! // so that even if the combinator is rejected,
//! // the whole combinator will still be accepted with 0 bytes digested
//! let _: Combinator<_> = exact("true") * (..);
//! let _: Combinator<_> = exact("true") * (..=3);
//!
//! // repeating for at most 0 times will
//! // always accept 0 bytes without executing the combinator.
//! // you shouldn't use this for most cases
//! let _: Combinator<_> = exact("true") * 0;
//! let _: Combinator<_> = exact("true") * (..1);
//! let _: Combinator<_> = exact("true") * (..=0);
//! ```
//! ## Decorator
//! You can use combinator decorators to modify the behavior of a combinator.
//! Unlike combining combinators, decorators won't change the digested content:
//! ```
//! # use whitehole::combinator::{Combinator, exact};
//! // make the combinator optional
//! let _: Combinator<_> = exact("true").optional();
//! ```
//! See [`Combinator`]'s methods for more decorators.

mod common;
mod decorator;
mod input;
mod output;

pub mod operator;

use std::marker::PhantomData;

pub use common::*;
pub use decorator::*;
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

/// Implement [`Parse`] for plain closures.
/// Currently this struct is required to constrain generic params.
/// Maybe removed in the future, thus private.
/// TODO: remove this.
#[derive(Debug, Clone, Copy)]
struct Closure<Kind, State, Heap, T> {
  closure: T,
  _phantom: PhantomData<(Kind, State, Heap)>,
}

impl<Kind, State, Heap, T> Closure<Kind, State, Heap, T> {
  #[inline]
  fn new(closure: T) -> Self {
    Self {
      closure,
      _phantom: PhantomData,
    }
  }
}

impl<
    Kind,
    State,
    Heap,
    T: for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Kind>>,
  > Parse for Closure<Kind, State, Heap, T>
{
  type Kind = Kind;
  type State = State;
  type Heap = Heap;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    (self.closure)(input)
  }
}

/// Wrap a [`Parse`] implementor and provide composition operators.
#[derive(Debug, Clone, Copy)]
pub struct Combinator<T> {
  parser: T,
}

impl<T> Combinator<T> {
  /// Create a new instance.
  #[inline]
  pub fn new(parser: T) -> Self {
    Self { parser }
  }

  // TODO
  // #[inline]
  // pub fn collapse(
  //   self,
  // ) -> Combinator<impl Parse> {
  //   self
  // }
}

/// Wrap a closure to create a [`Combinator`].
/// TODO: better signature?
#[inline]
pub fn wrap<Kind, State, Heap>(
  parse: impl for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Kind>>,
) -> Combinator<impl Parse<State = State, Heap = Heap, Kind = Kind>> {
  Combinator {
    parser: Closure::new(parse),
  }
}

impl<T: Parse> Parse for Combinator<T> {
  type Kind = T::Kind;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
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
