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

mod builder;
mod decorator;
mod provided;

pub mod ops;

pub use builder::*;
pub use decorator::*;
pub use provided::*;

use crate::parse::{Input, Output, Parse};
use std::marker::PhantomData;

/// Wrap a [`Parse`] implementor and provide composition operators.
#[derive(Debug, Clone, Copy)]
pub struct Combinator<Kind, State, Heap, T> {
  parser: T,
  _phantom: PhantomData<(Kind, State, Heap)>,
}

/// `C!(Kind, State, Heap)` will be expanded to `Combinator<Kind, State, Heap, impl Parse<Kind, State, Heap>>`.
#[macro_export]
macro_rules! C {
  ($kind:ty, $state:ty, $heap:ty) => {
    $crate::combinator::Combinator<$kind, $state, $heap, impl $crate::parse::Parse<$kind, $state, $heap>>
  };
}

impl<Kind, State, Heap, T> Combinator<Kind, State, Heap, T> {
  /// Create a new instance.
  #[inline]
  pub fn new(parser: T) -> Self {
    Self {
      parser,
      _phantom: PhantomData,
    }
  }

  // TODO
  #[inline]
  pub fn collapse(self) -> C!(Kind, State, Heap)
  where
    T: Parse<Kind, State, Heap>,
  {
    self
  }
}

/// Wrap a closure to create a [`Combinator`].
/// TODO: better signature?
#[inline]
pub fn wrap<Kind, State, Heap>(
  parse: impl for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Kind>>,
) -> C!(Kind, State, Heap) {
  Combinator::new(parse)
}

impl<Kind, State, Heap, T: Parse<Kind, State, Heap>> Parse<Kind, State, Heap>
  for Combinator<Kind, State, Heap, T>
{
  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Kind>> {
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
