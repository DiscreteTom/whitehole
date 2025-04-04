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
//! // similar to
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
//! # Accumulate Values
//! ## To an Array
//! If the repetition value is known at compile time,
//! you can use `* [v; len]` to accumulate the values to an array.
//! ```
//! # use whitehole::{combinator::next, parser::Parser};
//! let entry = {
//!   // accept one ascii digit at a time
//!   next(|c| c.is_ascii_digit())
//!     // convert the char to a number
//!     .select(|accepted| accepted.content().as_bytes()[0] - b'0')
//!     // repeat for 3 times, accumulate the values to an array
//!     * [0; 3]
//! };
//!
//! // parse "123"
//! assert_eq!(
//!   Parser::builder().entry(entry).build("123").next().unwrap().value,
//!   [1, 2, 3]
//! )
//! ```
//! The initial value of the array doesn't matter,
//! since the output value will be initialized by [`std::mem::zeroed`]
//! and filled by actual values during parsing.
//! ## Ad-hoc Accumulator
//! You can use [`Combinator::fold`]
//! to specify an ad-hoc accumulator after performing `*`.
//! ```
//! # use whitehole::{combinator::next, parser::Parser};
//! let entry = {
//!   // accept one ascii digit at a time
//!   next(|c| c.is_ascii_digit())
//!     // convert the char to a number
//!     .select(|accepted| accepted.content().as_bytes()[0] - b'0')
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
//! ## To the Heap
//! If your accumulator requires heap allocation,
//! each time the combinator is executed, the accumulator will be re-allocated and dropped.
//! That's not efficient.
//!
//! To optimize the performance,
//! you can fold the values to [`Parser::heap`](crate::parser::Parser::heap) to prevent re-allocation.
//! ```
//! # use whitehole::{combinator::contextual, parser::Parser};
//!
//! // generate contextual combinators
//! contextual!(Vec<i32>, ());
//!
//! let entry = {
//!   // eat one char, accumulate some value in the heap
//!   take(1).then(|accepted| accepted.heap.push(1))
//!     // repeat for 1 or more times
//!     * (1..)
//! }.prepare(|input| input.heap.clear()); // clear the vec before executing this combinator
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
//! Or with array accumulator
//! ```
//! # use whitehole::{combinator::eat, parser::Parser};
//! let entry = (eat('a').bind(1) * [0; 3]).sep(',');
//! assert_eq!(
//!   Parser::builder().entry(entry).build("a,a,a").next().unwrap().value,
//!   [1, 1, 1]
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
  mem::zeroed,
  ops::{self, RangeFrom},
  slice::SliceIndex,
};

/// An [`Action`] created by the `*` operator.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Mul<Lhs, Rhs, Sep = NoSep<Lhs>, Init = fn(), Fold = fn((), ())> {
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
      sep: NoSep::new(),
      init: || (),
      fold: |_, _| (),
    }
  }
}

impl<Lhs: Action, Rhs: Repeat> ops::Mul<Rhs> for Combinator<Lhs> {
  type Output = Combinator<Mul<Lhs, Rhs, NoSep<Lhs>>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul::new(self.action, rhs))
  }
}

impl<Lhs: Action, const N: usize> ops::Mul<[Lhs::Value; N]> for Combinator<Lhs> {
  type Output = Combinator<Mul<Lhs, [Lhs::Value; N], NoSep<Lhs>>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  fn mul(self, rhs: [Lhs::Value; N]) -> Self::Output {
    Self::Output::new(Mul::new(self.action, rhs))
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

unsafe impl<
    Lhs: Action<Text: Digest>,
    const N: usize,
    Sep: Action<Text = Lhs::Text, State = Lhs::State, Heap = Lhs::Heap>,
  > Action for Mul<Lhs, [Lhs::Value; N], Sep>
where
  RangeFrom<usize>: SliceIndex<Lhs::Text, Output = Lhs::Text>,
{
  type Text = Lhs::Text;
  type State = Lhs::State;
  type Heap = Lhs::Heap;
  type Value = [Lhs::Value; N];

  #[inline]
  fn exec(
    &self,
    mut input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let mut output: Output<[<Lhs as Action>::Value; N]> = Output {
      // SAFETY: if N is not 0, the zeroed value will be override by actual values,
      // and if N is 0, the zeroed value will be a valid empty slice.
      value: unsafe { zeroed() },
      digested: 0,
    };

    let mut digested_with_sep = 0;
    for i in 0..N {
      let value_output = self.lhs.exec(
        input.reborrow_with(&unsafe { input.instant.to_digested_unchecked(digested_with_sep) }),
      )?;
      // SAFETY: `i` must be in `0..N`
      debug_assert!(i < N);
      // TODO: what if the Value is Drop?
      *unsafe { output.value.get_unchecked_mut(i) } = value_output.value;
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - digested_with_sep > value_output.digested);
      output.digested = unsafe { digested_with_sep.unchecked_add(value_output.digested) };

      // SAFETY: `i` must be smaller than `N` and `N` is a valid usize
      if unsafe { i.unchecked_add(1) } == N {
        // skip the last separator if `N` is reached
        break;
      }

      let sep_output = self.sep.exec(
        input.reborrow_with(&unsafe { input.instant.to_digested_unchecked(output.digested) }),
      )?;
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - output.digested > sep_output.digested);
      digested_with_sep = unsafe { output.digested.unchecked_add(sep_output.digested) };
    }

    Some(output)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    action::{Action, Input, Output},
    combinator::{bytes, take},
    digest::Digest,
    instant::Instant,
  };
  use std::{fmt::Debug, ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text = Text, State = (), Heap = (), Value = ()>,
    input: &Text,
    expected: Option<usize>,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action.exec(Input {
        instant: &Instant::new(input),
        state: &mut (),
        heap: &mut ()
      }),
      expected.map(|digested| Output {
        value: (),
        digested,
      })
    )
  }

  #[test]
  fn combinator_mul_usize() {
    let accepter = || take(1);
    let accepter_b = || bytes::take(1);
    let rejecter = || take(0).reject(|_| true);
    let rejecter_b = || bytes::take(0).reject(|_| true);

    // normal
    helper(accepter() * 3, "1234", Some(3));
    helper(accepter_b() * 3, b"1234", Some(3));

    // reject if not enough repetitions
    helper(accepter() * 3, "12", None);
    helper(accepter_b() * 3, b"12", None);

    // reject with rejector
    helper(rejecter() * 3, "123", None);
    helper(rejecter_b() * 3, b"123", None);

    // repeat for 0 times will always accept with 0 bytes digested
    helper(accepter() * 0, "123", Some(0));
    helper(accepter_b() * 0, b"123", Some(0));
    // even with rejecter
    helper(rejecter() * 0, "123", Some(0));
    helper(rejecter_b() * 0, b"123", Some(0));
  }

  #[test]
  fn combinator_mul_range() {
    let accepter = || take(1);
    let accepter_b = || bytes::take(1);
    let rejecter = || take(0).reject(|_| true);
    let rejecter_b = || bytes::take(0).reject(|_| true);

    // normal
    helper(accepter() * (2..4), "1234", Some(3));
    helper(accepter_b() * (2..4), b"1234", Some(3));

    // reject if not enough repetitions
    helper(accepter() * (2..4), "1", None);
    helper(accepter_b() * (2..4), b"1", None);

    // reject with rejector
    helper(rejecter() * (2..4), "123", None);
    helper(rejecter_b() * (2..4), b"123", None);

    // repeat for 0 times will always accept with 0 bytes digested
    helper(accepter() * (0..1), "123", Some(0));
    helper(accepter_b() * (0..1), b"123", Some(0));
    // even with rejecter
    helper(rejecter() * (0..1), "123", Some(0));
    helper(rejecter_b() * (0..1), b"123", Some(0));
  }

  #[test]
  fn combinator_mul_range_from() {
    let accepter = || take(1);
    let accepter_b = || bytes::take(1);
    let rejecter = || take(0).reject(|_| true);
    let rejecter_b = || bytes::take(0).reject(|_| true);

    // normal
    helper(accepter() * (2..), "1234", Some(4));
    helper(accepter_b() * (2..), b"1234", Some(4));

    // reject if not enough repetitions
    helper(accepter() * (2..), "1", None);
    helper(accepter_b() * (2..), b"1", None);

    // reject with rejector
    helper(rejecter() * (2..), "123", None);
    helper(rejecter_b() * (2..), b"123", None);

    // repeat for 0 times will always accept with 0 bytes digested
    // even with rejecter
    helper(rejecter() * (0..), "123", Some(0));
    helper(rejecter_b() * (0..), b"123", Some(0));
  }

  #[test]
  fn combinator_mul_range_full() {
    let accepter = || take(1);
    let accepter_b = || bytes::take(1);
    let rejecter = || take(0).reject(|_| true);
    let rejecter_b = || bytes::take(0).reject(|_| true);

    // normal
    helper(accepter() * (..), "1234", Some(4));
    helper(accepter_b() * (..), b"1234", Some(4));

    // repeat for 0 times will always accept with 0 bytes digested
    // even with rejecter
    helper(rejecter() * (..), "123", Some(0));
    helper(rejecter_b() * (..), b"123", Some(0));
  }

  #[test]
  fn combinator_mul_range_inclusive() {
    let accepter = || take(1);
    let accepter_b = || bytes::take(1);
    let rejecter = || take(0).reject(|_| true);
    let rejecter_b = || bytes::take(0).reject(|_| true);

    // normal
    helper(accepter() * (2..=3), "1234", Some(3));
    helper(accepter_b() * (2..=3), b"1234", Some(3));

    // reject if not enough repetitions
    helper(accepter() * (2..=3), "1", None);
    helper(accepter_b() * (2..=3), b"1", None);

    // reject with rejector
    helper(rejecter() * (2..=3), "123", None);
    helper(rejecter_b() * (2..=3), b"123", None);

    // repeat for 0 times will always accept with 0 bytes digested
    helper(accepter() * (0..=0), "123", Some(0));
    helper(accepter_b() * (0..=0), b"123", Some(0));
    // even with rejecter
    helper(rejecter() * (0..=0), "123", Some(0));
    helper(rejecter_b() * (0..=0), b"123", Some(0));
  }

  #[test]
  fn combinator_mul_range_to() {
    let accepter = || take(1);
    let accepter_b = || bytes::take(1);
    let rejecter = || take(0).reject(|_| true);
    let rejecter_b = || bytes::take(0).reject(|_| true);

    // normal
    helper(accepter() * (..4), "1234", Some(3));
    helper(accepter_b() * (..4), b"1234", Some(3));

    // repeat for 0 times will always accept with 0 bytes digested
    helper(accepter() * (..1), "123", Some(0));
    helper(accepter_b() * (..1), b"123", Some(0));
    // even with rejecter
    helper(rejecter() * (..1), "123", Some(0));
    helper(rejecter_b() * (..1), b"123", Some(0));
  }

  #[test]
  fn combinator_mul_range_to_inclusive() {
    let accepter = || take(1);
    let accepter_b = || bytes::take(1);
    let rejecter = || take(0).reject(|_| true);
    let rejecter_b = || bytes::take(0).reject(|_| true);

    // normal
    helper(accepter() * (2..=3), "1234", Some(3));
    helper(accepter_b() * (2..=3), b"1234", Some(3));

    // reject if not enough repetitions
    helper(accepter() * (2..=3), "1", None);
    helper(accepter_b() * (2..=3), b"1", None);

    // reject with rejector
    helper(rejecter() * (2..=3), "123", None);
    helper(rejecter_b() * (2..=3), b"123", None);

    // repeat for 0 times will always accept with 0 bytes digested
    helper(accepter() * (0..=0), "123", Some(0));
    helper(accepter_b() * (0..=0), b"123", Some(0));
    // even with rejecter
    helper(rejecter() * (0..=0), "123", Some(0));
    helper(rejecter_b() * (0..=0), b"123", Some(0));
  }

  #[test]
  fn combinator_mul_array() {
    fn helper<Text: ?Sized + Digest, Value: PartialEq + Debug>(
      action: impl Action<Text = Text, State = (), Heap = (), Value = Value>,
      input: &Text,
      expected: Option<Output<Value>>,
    ) where
      RangeFrom<usize>: SliceIndex<Text, Output = Text>,
    {
      assert_eq!(
        action.exec(Input {
          instant: &Instant::new(input),
          state: &mut (),
          heap: &mut ()
        }),
        expected
      )
    }

    let accepter = || take(1).select(|accepted| accepted.content().as_bytes()[0] - b'0');
    let accepter_b = || bytes::take(1).select(|accepted| accepted.content()[0] - b'0');
    let rejecter = || accepter().reject(|_| true);
    let rejecter_b = || accepter_b().reject(|_| true);

    // normal
    helper(
      accepter() * [0; 3],
      "123",
      Some(Output {
        value: [1, 2, 3],
        digested: 3,
      }),
    );
    helper(
      accepter_b() * [0; 3],
      b"123",
      Some(Output {
        value: [1, 2, 3],
        digested: 3,
      }),
    );

    // reject if not enough repetitions
    helper(accepter() * [0; 3], "12", None);
    helper(accepter_b() * [0; 3], b"12", None);

    // reject with rejector
    helper(rejecter() * [0; 3], "123", None);
    helper(rejecter_b() * [0; 3], b"123", None);

    // repeat for 0 times will always accept with 0 bytes digested
    helper(
      accepter() * [0; 0],
      "123",
      Some(Output {
        value: [],
        digested: 0,
      }),
    );
    helper(
      accepter_b() * [0; 0],
      b"123",
      Some(Output {
        value: [],
        digested: 0,
      }),
    );
    // even with rejecter
    helper(
      rejecter_b() * [0; 0],
      b"123",
      Some(Output {
        value: [],
        digested: 0,
      }),
    );
  }
}
