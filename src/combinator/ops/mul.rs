//! Overload `*` operator for [`Combinator`](crate::combinator::Combinator).
//!
//! `Combinator * Repeat` will create a new combinator to repeat the original combinator
//! with the given [`Repeat`] range.
//! The new combinator will return the output with the [`Fold`]-ed value
//! and the rest of the input text after the last repetition is executed,
//! or reject if the repetition is not satisfied.
//!
//! `0` is a valid repetition value.
//! # Basics
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
//! ## Ad-hoc accumulator
//! ```
//! # use whitehole::{combinator::next, action::{Input, Action}, instant::Instant};
//! let combinator = {
//!   // accept one ascii digit at a time
//!   next(|c| c.is_ascii_digit())
//!     // convert the char to a number
//!     .select(|ctx| ctx.input().instant().rest().chars().next().unwrap() as usize - '0' as usize)
//!     // repeat for 1 or more times
//!     * (1..)
//! }
//! // init accumulator with 0, and fold values
//! .fold(|| 0 as usize, |value, acc| acc * 10 + value);
//!
//! // parse "123" to 123
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("123"), &mut (), &mut ())).unwrap().value,
//!   123
//! )
//! ```
//! ## Fold to heap
//! If your accumulator requires heap allocation,
//! each time the combinator is executed, the accumulator will be re-allocated and dropped.
//! That's not efficient.
//!
//! To optimize the performance,
//! you can fold the values to [`Input::heap`](crate::action::Input::heap) to prevent re-allocation.
//! ```
//! # use whitehole::{combinator::eat, action::{Input, Action}, instant::Instant};
//! let combinator = {
//!   // eat one char, accumulate the start index in `input.heap`
//!   eat(1).then(|mut ctx| ctx.heap().push(ctx.start()))
//!     // repeat for 1 or more times
//!     * (1..)
//! }.prepare(|input| input.heap.clear()); // clear the vec before executing this combinator
//!
//! // create a re-usable heap
//! let mut heap = vec![];
//! combinator.exec(Input::new(Instant::new("123"), &mut (), &mut heap));
//! assert_eq!(heap, vec![0, 1, 2]);
//! ```
//! # Separator
//! You can use [`Combinator::sep`](crate::combinator::Combinator::sep)
//! to specify an other combinator as the separator after performing `*`.
//! ```
//! # use whitehole::{combinator::eat, action::{Input, Action}, instant::Instant};
//! let combinator = (eat('a') * (1..)).sep(',');
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ())).unwrap().digested,
//!   5
//! )
//! ```
//! You can fold the values with the separator.
//! ```
//! # use whitehole::{combinator::{ops::mul::Fold, eat}, action::{Input, Action}, instant::Instant};
//! let combinator = (eat('a').bind(1) * (1..)).fold(|| 0, |v, acc| acc + v).sep(',');
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ())).unwrap().value,
//!   3
//! );
//! ```
//! See [`Combinator::sep`] for more information.
mod fold;
mod repeat;
mod sep;

pub use repeat::*;

use crate::combinator::Combinator;
use std::ops;

/// An [`Action`](crate::action::Action) created by the `*` operator.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Mul<Lhs, Rhs, Sep = (), Init = fn(), Fold = fn((), ())> {
  lhs: Lhs,
  rhs: Rhs,
  sep: Sep,
  init: Init,
  fold: Fold,
}

impl<Lhs, Rhs> Mul<Lhs, Rhs> {
  #[inline]
  const fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self {
      lhs,
      rhs,
      sep: (),
      init: || (),
      fold: |_, _| (),
    }
  }
}

impl<Lhs, Rhs: Repeat> ops::Mul<Rhs> for Combinator<Lhs> {
  type Output = Combinator<Mul<Lhs, Rhs>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul::new(self.action, rhs))
  }
}
