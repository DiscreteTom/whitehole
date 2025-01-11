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
//! ## Inline Fold
//! For simple cases, you can accumulate values inline.
//! ```
//! # use whitehole::{combinator::next, action::{Input, Action}, instant::Instant};
//! let combinator =
//!   // accept one ascii digit at a time
//!   next(|c| c.is_ascii_digit())
//!     // convert the char to a number
//!     .select(|ctx| ctx.input().instant().rest().chars().next().unwrap() as usize - '0' as usize)
//!     // init accumulator with 0, and fold values
//!     .fold(|| 0 as usize, |value, acc, _| acc * 10 + value)
//!     // repeat for 1 or more times
//!     * (1..);
//!
//! // parse "123" to 123
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("123"), &mut (), &mut ())).unwrap().value,
//!   123
//! )
//! ```
//! ## Fold with Custom Type
//! If you want to re-use the fold logic, you can implement [`Fold`] for a custom type.
//! ```
//! # use whitehole::{combinator::{ops::mul::Fold, next}, action::{Input, Action}, instant::Instant};
//! // since you can't implement `Fold` for `usize` directly,
//! // wrap it in a new-type
//! struct Usize(usize);
//!
//! impl<State, Heap> Fold<State, Heap> for Usize {
//!   type Output = usize;
//!
//!   fn fold(self, acc: Self::Output, _input: Input<&mut State, &mut Heap>) -> Self::Output {
//!     acc * 10 + self.0
//!   }
//! }
//!
//! let combinator =
//!   // accept one ascii digit at a time
//!   next(|c| c.is_ascii_digit())
//!     // convert the char to a number, wrapped in `Usize`
//!     .select(|ctx| Usize(ctx.input().instant().rest().chars().next().unwrap() as usize - '0' as usize))
//!     // repeat for 1 or more times, fold `Usize` to `usize`
//!     * (1..);
//!
//! // parse "123" to 123
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("123"), &mut (), &mut ())).unwrap().value,
//!   123
//! )
//! ```
//! ## Fold to Heap
//! If your accumulator requires heap allocation,
//! each time the combinator is executed, the accumulator will be re-allocated and dropped.
//! That's not efficient.
//!
//! To optimize the performance,
//! you can fold the values to [`Input::heap`](crate::action::Input::heap) to prevent re-allocation.
//! ```
//! # use whitehole::{combinator::eat, action::{Input, Action}, instant::Instant};
//! let combinator = {
//!   // eat one char, use the start index as the value
//!   eat(1).select(|ctx| ctx.start())
//!     // fold values to a vec, store values in `input.heap`
//!     .fold(|| {}, |value, _acc, input: Input<_, &mut Vec<_>>| input.heap.push(value))
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
//! // inline fold
//! # use whitehole::{combinator::{ops::mul::Fold, eat}, action::{Input, Action}, instant::Instant};
//! let combinator = (eat('a').bind(1).fold(|| 0, |v, acc, _| acc + v) * (1..)).sep(',');
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ())).unwrap().value,
//!   3
//! );
//!
//! // with custom type
//! #[derive(Clone)]
//! struct Usize(usize);
//! impl<State, Heap> Fold<State, Heap> for Usize {
//!   type Output = usize;
//!   fn fold(self, acc: Self::Output, _input: Input<&mut State, &mut Heap>) -> Self::Output {
//!     acc + self.0
//!   }
//! }
//! let combinator = (eat('a').bind(Usize(1)) * (1..)).sep(',');
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ())).unwrap().value,
//!   3
//! )
//! ```
//! See [`Combinator::sep`](crate::combinator::Combinator::sep) for more information.
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

// use a macro to ensure inline
macro_rules! impl_mul {
  ($input:ident, $repeat:expr, $init:expr, $fold:expr, $action:expr) => {{
    let mut repeated = 0;
    let mut output = Output {
      value: $init(),
      digested: 0,
    };

    while unsafe { $repeat.validate(repeated) } {
      let Some(next_output) = $action.exec(shift_input!($input, output.digested)) else {
        break;
      };

      output.value = $fold(next_output.value, output.value, $input.reborrow());
      repeated += 1;
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - output.digested > next_output.digested);
      output.digested = unsafe { output.digested.unchecked_add(next_output.digested) };
    }

    $repeat.accept(repeated).then_some(output)
  }};
}
// https://github.com/rust-lang/rust-clippy/issues/12808
#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use impl_mul;
