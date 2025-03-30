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
//! You can use [`Combinator::fold`]
//! to specify an ad-hoc accumulator after performing `*`.
//! ```
//! # use whitehole::{combinator::next, parser::Parser};
//! let entry = {
//!   // accept one ascii digit at a time
//!   next(|c| c.is_ascii_digit())
//!     // convert the char to a number
//!     .select(|accept, _| accept.instant().rest().chars().next().unwrap() as usize - '0' as usize)
//!     // repeat for 1 or more times
//!     * (1..)
//! }
//! // init accumulator with 0, and fold values
//! .fold(|| 0 as usize, |acc, value| acc * 10 + value);
//!
//! // parse "123" to 123
//! assert_eq!(
//!   Parser::builder().entry(entry).build("123").next().unwrap().value,
//!   123
//! )
//! ```
//! ## Fold to heap
//! If your accumulator requires heap allocation,
//! each time the combinator is executed, the accumulator will be re-allocated and dropped.
//! That's not efficient.
//!
//! To optimize the performance,
//! you can fold the values to [`Context::heap`] to prevent re-allocation.
//! ```
//! # use whitehole::{combinator::take, parser::Parser};
//! let entry = {
//!   // eat one char, accumulate some value in `ctx.heap`
//!   take(1).then(|_, ctx| Vec::push(ctx.heap, 1))
//!     // repeat for 1 or more times
//!     * (1..)
//! }.prepare(|_, ctx| ctx.heap.clear()); // clear the vec before executing this combinator
//!
//! // create a re-usable heap
//! let mut parser = Parser::builder().heap(vec![]).entry(entry).build("123");
//! parser.next();
//! assert_eq!(parser.heap, vec![1, 1, 1]);
//! ```
//! # Separator
//! You can use [`Combinator::sep`]
//! to specify an other combinator as the separator after performing `*`.
//! ```
//! # use whitehole::{combinator::eat, parser::Parser};
//! let entry = (eat('a') * (1..)).sep(',');
//! assert_eq!(
//!   Parser::builder().entry(entry).build("a,a,a").next().unwrap().digested,
//!   5
//! )
//! ```
//! You can use [`Combinator::sep`] with [`Combinator::fold`]:
//! ```
//! # use whitehole::{combinator::eat, parser::Parser};
//! let entry = (eat('a').bind(1) * (1..)).sep(',').fold(|| 0, |v, acc| acc + v);
//! assert_eq!(
//!   Parser::builder().entry(entry).build("a,a,a").next().unwrap().value,
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
  instant::Instant,
};
use std::{
  ops::{self, RangeFrom},
  slice::SliceIndex,
};

/// An [`Action`] created by the `*` operator.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Mul<Lhs, Rhs, Sep, Init = fn(), Fold = fn((), ())> {
  lhs: Lhs,
  rhs: Rhs,
  sep: Sep,
  init: Init,
  fold: Fold,
}

// TODO
// impl<Lhs, Rhs> Mul<Lhs, Rhs> {
//   #[inline]
//   const fn new(lhs: Lhs, rhs: Rhs) -> Self {
//     Self {
//       lhs,
//       rhs,
//       sep: NoSep::new(),
//       init: || (),
//       fold: |_, _| (),
//     }
//   }
// }

impl<Lhs: Action, Rhs: Repeat> ops::Mul<Rhs> for Combinator<Lhs> {
  type Output = Combinator<Mul<Lhs, Rhs, NoSep<Lhs::Text, Lhs::State, Lhs::Heap>>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul {
      lhs: self.action,
      rhs,
      sep: NoSep::new(),
      init: || (),
      fold: |_, _| (),
    })
  }
}

unsafe impl<
    Lhs: Action<Text: Digest>,
    Rhs: Repeat,
    Sep: Action<Text = Lhs::Text, State = Lhs::State, Heap = Lhs::Heap>,
    Acc,
    Init: Fn() -> Acc,
    Fold: Fn(Acc, Lhs::Value) -> Acc,
  > Action for Mul<Lhs, Rhs, Sep, Init, Fold>
where
  RangeFrom<usize>: SliceIndex<Lhs::Text, Output = Lhs::Text>,
{
  type Text = Lhs::Text;
  type State = Lhs::State;
  type Heap = Lhs::Heap;
  type Value = Acc;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let mut repeated = 0;
    let mut output = Output {
      value: (self.init)(),
      digested: 0,
    };

    let mut digested_with_sep = 0;
    while unsafe { self.rhs.validate(repeated) } {
      let Some(value_output) = self.lhs.exec(
        input.reborrow_with(&unsafe { input.instant.to_digested_unchecked(digested_with_sep) }),
      ) else {
        break;
      };
      repeated += 1;
      output.value = (self.fold)(output.value, value_output.value);
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - digested_with_sep > value_output.digested);
      output.digested = unsafe { digested_with_sep.unchecked_add(value_output.digested) };

      let Some(sep_output) = self.sep.exec(
        input.reborrow_with(&unsafe { input.instant.to_digested_unchecked(output.digested) }),
      ) else {
        break;
      };
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - output.digested > sep_output.digested);
      digested_with_sep = unsafe { output.digested.unchecked_add(sep_output.digested) };
    }

    self.rhs.accept(repeated).then_some(output)
  }
}
