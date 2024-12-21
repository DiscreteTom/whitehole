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
//! # use whitehole::{combinator::eat, Combinator};
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
//!
//! // repeat with another combinator as the separator
//! # t(
//! eat("true") * (1.., eat(','))
//! # );
//! // you can use a String, a &str or a char as the separator
//! # t(
//! eat("true") * (1.., ", ".to_string())
//! # );
//! # t(
//! eat("true") * (1.., ", ")
//! # );
//! # t(
//! eat("true") * (1.., ',')
//! # );
//! ```
//! If there is at least one repetition, then the separator is allowed to be the last match.
//! E.g. `eat('a') * (1.., eat(','))` will accept `"a"`, `"a,"`, `"a,a"` but reject `","`.
//! You have to check if the last match is a separator by yourself.
//! # Fold Values
//! ## Inline Fold
//! For simple cases, you can accumulate values inline.
//! ```
//! # use whitehole::{combinator::next, action::{Input, Action}};
//! let combinator =
//!   // accept one ascii digit at a time
//!   next(|c| c.is_ascii_digit())
//!     // convert the char to a number
//!     .select(|ctx| ctx.input.next() as usize - '0' as usize)
//!     // repeat for 1 or more times, init accumulator with 0, and fold values
//!     * (1.., || 0 as usize, |value, acc| acc * 10 + value);
//!
//! // parse "123" to 123
//! assert_eq!(
//!   combinator.exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()).unwrap().value,
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
//!     .select(|ctx| Usize(ctx.input.next() as usize - '0' as usize))
//!     // repeat for 1 or more times, fold `Usize` to `usize`
//!     * (1..);
//!     // equals to: `* (1.., Usize::Output::default, Usize::fold)`
//!
//! // parse "123" to 123
//! assert_eq!(
//!   combinator.exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()).unwrap().value,
//!   123
//! )
//! ```
mod fold;
mod inline;
mod repeat;

pub use fold::*;
pub use repeat::*;

use crate::combinator::{Action, Input, Output};

/// An [`Action`] created by the `*` operator.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
#[derive(Debug, Clone, Copy)]
pub struct Mul<Lhs, Rhs> {
  pub lhs: Lhs,
  pub rhs: Rhs,
}

impl<Lhs, Rhs> Mul<Lhs, Rhs> {
  #[inline]
  pub const fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self { lhs, rhs }
  }
}

#[inline]
fn impl_mul<'text, Value, State, Heap, Acc>(
  lhs: &impl Action<Value = Value, State = State, Heap = Heap>,
  range: &impl Repeat,
  init: impl Fn() -> Acc,
  folder: impl Fn(Value, Acc) -> Acc,
  input: &mut Input<'text, &mut State, &mut Heap>,
) -> Option<Output<'text, Acc>> {
  let mut repeated = 0;
  let mut output = Output {
    value: init(),
    rest: input.rest(),
  };

  while range.validate(repeated) {
    let Some(next_output) = input
      .reload(output.rest)
      .and_then(|mut input| lhs.exec(&mut input))
    else {
      break;
    };
    output.rest = next_output.rest;
    output.value = folder(next_output.value, output.value);
    repeated += 1;
  }

  range.accept(repeated).then_some(output)
}

#[inline]
fn impl_mul_with_sep<'text, Value, State, Heap, Acc>(
  lhs: &impl Action<Value = Value, State = State, Heap = Heap>,
  range: &impl Repeat,
  sep: &impl Action<Value = (), State = State, Heap = Heap>,
  init: impl Fn() -> Acc,
  folder: impl Fn(Value, Acc) -> Acc,
  input: &mut Input<'text, &mut State, &mut Heap>,
) -> Option<Output<'text, Acc>> {
  let mut repeated = 0;
  let mut output = Output {
    value: init(),
    rest: input.rest(),
  };

  while range.validate(repeated) {
    let Some(next_output) = input
      .reload(output.rest)
      .and_then(|mut input| lhs.exec(&mut input))
    else {
      break;
    };
    repeated += 1;
    output.rest = next_output.rest;
    output.value = folder(next_output.value, output.value);
    let Some(next_output) = input
      .reload(next_output.rest)
      .and_then(|mut input| sep.exec(&mut input))
    else {
      break;
    };
    output.rest = next_output.rest;
  }

  range.accept(repeated).then_some(output)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::{wrap, Input, Output};

  #[derive(Debug)]
  struct MyValue(usize);
  impl Fold for MyValue {
    type Output = usize;
    fn fold(self, current: Self::Output) -> Self::Output {
      self.0 + current
    }
  }

  #[test]
  fn combinator_mul_usize() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * 3)
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    let n = 0;
    assert_eq!(
      (rejecter() * n).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    let n = 0;
    assert_eq!(
      (accepter() * n).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * 3).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );

    // overflow, reject
    assert!((accepter() * 4)
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..2))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..2)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..1)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 1,
        rest: "3"
      })
    );

    // too few, reject
    assert!((accepter() * (4..6))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_from() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );

    // too few, reject
    assert!((accepter() * (4..))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_full() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );
  }

  #[test]
  fn combinator_mul_range_inclusive() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..=3))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..=2)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..=0)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..=3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );

    // too few, reject
    assert!((accepter() * (4..=6))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_to() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..2)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..1)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 1,
        rest: "3"
      })
    );
  }

  #[test]
  fn combinator_mul_range_to_inclusive() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..=2)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..=0)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..=3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );
  }

  #[test]
  fn combinator_mul_with_sep() {
    let eat_char = |c| {
      wrap(move |input| {
        (input.next() == c).then(|| Output {
          value: (),
          rest: &input.rest()[1..],
        })
      })
    };
    let eat_a = || eat_char('a');
    let sep = || eat_char(',');

    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new(",", 0, &mut (), &mut ()).unwrap()),
      None
    );
    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new("a", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ""
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new("a,", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ""
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new("a,a", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ""
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new("a,,", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ","
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new("a,aa", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "a"
      })
    );

    // TODO: more tests
  }
}
