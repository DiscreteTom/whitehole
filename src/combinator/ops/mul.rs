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

/// An [`Action`](crate::action::Action) created by the `*` operator.
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    action::Action,
    combinator::{wrap, Input, Output},
  };

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
    let rejecter = || unsafe { wrap(|_| Option::<Output<()>>::None) };
    let accepter = || unsafe {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.start())))
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
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    let n = 0;
    assert_eq!(
      (accepter() * n).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * 3).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // overflow, reject
    assert!((accepter() * 4)
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range() {
    let rejecter = || unsafe { wrap(|_| Option::<Output<()>>::None) };
    let accepter = || unsafe {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.start())))
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
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..1)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 1,
        digested: 2
      })
    );

    // too few, reject
    assert!((accepter() * (4..6))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_from() {
    let rejecter = || unsafe { wrap(|_| Option::<Output<()>>::None) };
    let accepter = || unsafe {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.start())))
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
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // too few, reject
    assert!((accepter() * (4..))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_full() {
    let rejecter = || unsafe { wrap(|_| Option::<Output<()>>::None) };
    let accepter = || unsafe {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.start())))
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 3,
        digested: 3
      })
    );
  }

  #[test]
  fn combinator_mul_range_inclusive() {
    let rejecter = || unsafe { wrap(|_| Option::<Output<()>>::None) };
    let accepter = || unsafe {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.start())))
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
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..=0)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..=3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 3,
        digested: 3
      })
    );

    // too few, reject
    assert!((accepter() * (4..=6))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_to() {
    let rejecter = || unsafe { wrap(|_| Option::<Output<()>>::None) };
    let accepter = || unsafe {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.start())))
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..2)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..1)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 1,
        digested: 2
      })
    );
  }

  #[test]
  fn combinator_mul_range_to_inclusive() {
    let rejecter = || unsafe { wrap(|_| Option::<Output<()>>::None) };
    let accepter = || unsafe {
      wrap(|input| {
        input
          .digest(1)
          .map(|output| output.map(|_| MyValue(input.start())))
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..=2)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..=0)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        digested: 0,
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..=3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 3,
        digested: 3
      })
    );
  }
}
