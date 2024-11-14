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

pub use common::*;
pub use decorator::*;
pub use input::*;
pub use output::*;

/// This trait provides combinator decorators.
/// You can use [`impl_combinator!`] to implement this trait for your combinator.
pub trait Combinator {
  /// If the combinator is rejected, accept it with the default kind and zero digested.
  /// # Caveats
  /// This requires the `Kind` to implement [`Default`],
  /// thus usually used before setting a custom kind.
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// // bind a kind after calling `optional`
  /// combinator.optional().bind(MyKind::A)
  /// // instead of
  /// // combinator.bind(MyKind::A).optional()
  /// # ;}
  /// ```
  /// Or you can wrap `Kind` with [`Option`]:
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.bind(Some(MyKind::A)).optional()
  /// # ;}
  /// ```
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.optional()
  /// # ;}
  /// ```
  #[inline]
  fn optional(self) -> Optional<Self>
  where
    Self: Sized,
  {
    Optional::new(self)
  }
}

/// Provide the [`parse`](Parse::parse) method.
pub trait Parse<State = (), Heap = ()> {
  /// See [`Output::kind`].
  type Kind;

  /// Return [`None`] if the combinator is rejected.
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>>;
}

/// Implement [`Combinator`] and override [`Add`](std::ops::Add),
/// [`BitOr`](std::ops::BitOr), [`Mul`](std::ops::Mul) operators.
/// TODO: examples
#[macro_export]
macro_rules! impl_combinator {
  ($type:ty) => {
    impl_combinator!($type,);
  };
  ($type:ty, $($generic:ident),*) => {
    impl<$($generic),*> $crate::combinator::Combinator for $type {}

    impl<_Rhs, $($generic),*> std::ops::Mul<_Rhs> for $type {
      type Output = $crate::combinator::operator::mul::Mul<Self, _Rhs>;

      // TODO: fix doc link

      /// Repeat the combinator `rhs` times.
      /// Return the output with the [`Fold`]-ed kind value and the sum of the digested.
      ///
      /// See [`Fold`] for more information.
      fn mul(self, rhs: _Rhs) -> Self::Output {
        Self::Output::new(self, rhs)
      }
    }

    impl<_Rhs, $($generic),*> std::ops::BitOr<_Rhs> for $type {
      type Output = $crate::combinator::operator::bitor::BitOr<Self, _Rhs>;

      /// Try to parse with the left-hand side, if it fails, try the right-hand side.
      #[inline]
      fn bitor(self, rhs: _Rhs) -> Self::Output {
        Self::Output::new(self, rhs)
      }
    }

    impl<_Rhs, $($generic),*> std::ops::Add<_Rhs> for $type {
      type Output = $crate::combinator::operator::add::Add<Self, _Rhs>;

      /// Parse with the left-hand side, then parse with the right-hand side.
      /// Return the output with [`Concat`]-ed kind and the sum of the digested.
      #[inline]
      fn add(self, rhs: _Rhs) -> Self::Output {
        Self::Output::new(self, rhs)
      }
    }
  };
}

// #[cfg(test)]
// mod tests {
//   use super::*;

//   #[test]
//   fn combinator_parse() {
//     assert_eq!(
//       Combinator::boxed(|input| Some(Output {
//         kind: (),
//         rest: &input.rest()[1..]
//       }))
//       .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "23"
//       })
//     );
//   }
// }
