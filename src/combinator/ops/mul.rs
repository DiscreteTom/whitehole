//! Overload `*` operator for [`Combinator`](crate::combinator::Combinator).
//!
//! `Combinator * Repeat` will create a new combinator to repeat the original combinator
//! with the given [`Repeat`] range.
//! The new combinator will return the output with the [`Fold`]-ed value
//! and the rest of the input text after the last repetition is executed,
//! or reject if the repetition is not satisfied.
//!
//! `0` is a valid repetition range, which means the combinator is optional.
//! # Basics
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
//! ```
//! # Fold Values
//! ## Inline Fold
//! For simple cases, you can accumulate values inline.
//! ```
//! # use whitehole::{combinator::next, action::{Input, Action}};
//! let combinator =
//!   // accept one ascii digit at a time
//!   next(|c| c.is_ascii_digit())
//!     // convert the char to a number
//!     .select(|ctx| ctx.input().next() as usize - '0' as usize)
//!     // repeat for 1 or more times, init accumulator with 0, and fold values
//!     * (1.., || 0 as usize, |value, acc| acc * 10 + value);
//!
//! // parse "123" to 123
//! assert_eq!(
//!   combinator.exec(Input::new("123", 0, &mut (), &mut ()).unwrap()).unwrap().value,
//!   123
//! )
//! ```
//! ## Fold with Custom Type
//! If you want to re-use the fold logic, you can implement [`Fold`] for a custom type.
//! ```
//! # use whitehole::{combinator::{ops::mul::Fold, next}, action::{Input, Action}};
//! // since you can't implement `Fold` for `usize` directly,
//! // wrap it in a new-type
//! struct Usize(usize);
//!
//! impl Fold for Usize {
//!   type Output = usize;
//!
//!   fn fold(self, acc: Self::Output) -> Self::Output {
//!     acc * 10 + self.0
//!   }
//! }
//!
//! let combinator =
//!   // accept one ascii digit at a time
//!   next(|c| c.is_ascii_digit())
//!     // convert the char to a number, wrapped in `Usize`
//!     .select(|ctx| Usize(ctx.input().next() as usize - '0' as usize))
//!     // repeat for 1 or more times, fold `Usize` to `usize`
//!     * (1..);
//!     // equals to: `* (1.., Usize::Output::default, Usize::fold)`
//!
//! // parse "123" to 123
//! assert_eq!(
//!   combinator.exec(Input::new("123", 0, &mut (), &mut ()).unwrap()).unwrap().value,
//!   123
//! )
//! ```
//! # Separator
//! You can use [`Combinator::sep`](crate::combinator::Combinator::sep)
//! to specify an other combinator as the separator, then perform `*` on the pair.
//! ```
//! # use whitehole::{combinator::eat, action::{Input, Action}};
//! let combinator = eat('a').sep(',') * (1..);
//!
//! assert_eq!(
//!   combinator.exec(Input::new("a,a,a", 0, &mut (), &mut ()).unwrap()).unwrap().digested,
//!   5
//! )
//! ```
mod fold;
mod inline;
mod repeat;
mod sep;

pub use fold::*;
pub use repeat::*;
pub use sep::*;

/// An [`Action`](crate::action::Action) created by the `*` operator.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Mul<Lhs, Rhs> {
  lhs: Lhs,
  rhs: Rhs,
}

impl<Lhs, Rhs> Mul<Lhs, Rhs> {
  #[inline]
  const fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self { lhs, rhs }
  }
}
