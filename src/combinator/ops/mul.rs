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
//!     .select(|ctx| ctx.input().next() as usize - '0' as usize)
//!     // repeat for 1 or more times, init accumulator with 0, and fold values
//!     * (1.., || 0 as usize, |value, acc, _input: Input<&mut (), &mut ()>| acc * 10 + value);
//!
//! // parse "123" to 123
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap()).unwrap().value,
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
//!     .select(|ctx| Usize(ctx.input().next() as usize - '0' as usize))
//!     // repeat for 1 or more times, fold `Usize` to `usize`
//!     * (1..);
//!     // equals to: `* (1.., Usize::Output::default, Usize::fold)`
//!
//! // parse "123" to 123
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap()).unwrap().value,
//!   123
//! )
//! ```
//! ## Fold to Heap
//! You can fold the values to [`Input::heap`](crate::action::Input::heap) to prevent re-allocation.
//! ```
//! # use whitehole::{combinator::eat, action::{Input, Action}, instant::Instant};
//! let combinator = {
//!   // eat one char, use the start index as the value
//!   eat(1).select(|ctx| ctx.start())
//!     // repeat for 1 or more times, fold values to a vec, no need to init or use the accumulator
//!     * (1.., || {}, |value, _acc, input: Input<&mut (), &mut Vec<_>>| input.heap.push(value))
//! }.prepare(|input| input.heap.clear()); // clear the vec before executing this combinator
//!
//! // create a re-usable heap
//! let mut heap = vec![];
//! combinator.exec(Input::new(Instant::new("123"), &mut (), &mut heap).unwrap());
//! assert_eq!(heap, vec![0, 1, 2]);
//! ```
//! # Separator
//! You can use [`Combinator::sep`](crate::combinator::Combinator::sep)
//! to specify an other combinator as the separator, then perform `*` on the pair.
//! ```
//! # use whitehole::{combinator::eat, action::{Input, Action}, instant::Instant};
//! let combinator = eat('a').sep(',') * (1..);
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ()).unwrap()).unwrap().digested,
//!   5
//! )
//! ```
//! You can fold the values with the separator.
//! ```
//! // inline fold
//! # use whitehole::{combinator::{ops::mul::Fold, eat}, action::{Input, Action}, instant::Instant};
//! let combinator = eat('a').bind(1).sep(',') * (1.., || 0, |v, acc, _input: Input<&mut (), &mut ()>| acc + v);
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ()).unwrap()).unwrap().value,
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
//! let combinator = eat('a').bind(Usize(1)).sep(',') * (1..);
//! assert_eq!(
//!   combinator.exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ()).unwrap()).unwrap().value,
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
      let Some(next_output) =
        shift_input!($input, output.digested).and_then(|input| $action.exec(input))
      else {
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
pub(self) use impl_mul;

macro_rules! impl_mul_with_sep {
  ($input:ident, $repeat:expr, $init:expr, $fold:expr, $action:expr, $sep:expr) => {{
    let mut repeated = 0;
    let mut output = Output {
      value: $init(),
      digested: 0,
    };

    let mut digested_with_sep = 0;
    while unsafe { $repeat.validate(repeated) } {
      let Some(value_output) =
        shift_input!($input, digested_with_sep).and_then(|input| $action.exec(input))
      else {
        break;
      };
      repeated += 1;
      output.value = $fold(value_output.value, output.value, $input.reborrow());
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - digested_with_sep > value_output.digested);
      output.digested = unsafe { digested_with_sep.unchecked_add(value_output.digested) };

      let Some(sep_output) =
        shift_input!($input, output.digested).and_then(|input| $sep.exec(input))
      else {
        break;
      };
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - output.digested > sep_output.digested);
      digested_with_sep = unsafe { output.digested.unchecked_add(sep_output.digested) };
    }

    $repeat.accept(repeated).then_some(output)
  }};
}
pub(self) use impl_mul_with_sep;
