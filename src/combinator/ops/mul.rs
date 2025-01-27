//! Overload `*` operator for [`Combinator`].
//!
//! `Combinator * Repeat` will create a new combinator to repeat the original combinator
//! with the given [`Repeat`] range.
//! The new combinator will return the output with the folded value
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
//! # Fold values
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
//! you can fold the values to [`Input::heap`] to prevent re-allocation.
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
//! You can use [`Combinator::sep`]
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
pub use sep::*;

use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
  digest::Digest,
};
use std::ops;

/// An [`Action`] created by the `*` operator.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Mul<Lhs, Rhs, Sep = NoSep, Init = fn(), Fold = fn((), ())> {
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
      sep: NoSep,
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

unsafe impl<
    Text: ?Sized,
    State,
    Heap,
    Lhs: Action<Text, State, Heap>,
    Rhs: Repeat,
    Sep: Action<Text, State, Heap>,
    Acc,
    Init: Fn() -> Acc,
    Fold: Fn(Lhs::Value, Acc) -> Acc,
  > Action<Text, State, Heap> for Mul<Lhs, Rhs, Sep, Init, Fold>
where
  for<'a> &'a Text: Digest,
{
  type Value = Acc;

  #[inline]
  fn exec(&self, mut input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    let mut repeated = 0;
    let mut output = Output {
      value: (self.init)(),
      digested: 0,
    };

    let mut digested_with_sep = 0;
    while unsafe { self.rhs.validate(repeated) } {
      let Some(value_output) = self
        .lhs
        .exec(unsafe { input.shift_unchecked(digested_with_sep) })
      else {
        break;
      };
      repeated += 1;
      output.value = (self.fold)(value_output.value, output.value);
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - digested_with_sep > value_output.digested);
      output.digested = unsafe { digested_with_sep.unchecked_add(value_output.digested) };

      let Some(sep_output) = self
        .sep
        .exec(unsafe { input.shift_unchecked(output.digested) })
      else {
        break;
      };
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - output.digested > sep_output.digested);
      digested_with_sep = unsafe { output.digested.unchecked_add(sep_output.digested) };
    }

    self.rhs.accept(repeated).then_some(output)
  }
}
