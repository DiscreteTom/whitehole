//! Overload `*` operator for [`Combinator`].

use crate::combinator::{Combinator, Input, Output, Parse};
use std::ops::{self, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/// A helper trait to represent repetition when performing `*` on [`Combinator`]s.
///
/// Built-in implementations are provided for
/// [`usize`], [`Range<usize>`], [`RangeFrom<usize>`], [`RangeFull`],
/// [`RangeInclusive<usize>`], [`RangeTo<usize>`], and [`RangeToInclusive<usize>`].
pub trait Repeat {
  /// Check if the repetition should continue
  /// based on the current repeated times.
  fn validate(&self, repeated: usize) -> bool;

  /// Check if the repetition should be accepted
  /// based on the current repeated times.
  fn accept(&self, repeated: usize) -> bool;
}

impl Repeat for usize {
  #[inline]
  fn validate(&self, repeated: usize) -> bool {
    repeated < *self
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    repeated == *self
  }
}

impl Repeat for Range<usize> {
  #[inline]
  fn validate(&self, repeated: usize) -> bool {
    repeated + 1 < self.end
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeFrom<usize> {
  #[inline]
  fn validate(&self, _: usize) -> bool {
    true
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeFull {
  #[inline]
  fn validate(&self, _: usize) -> bool {
    true
  }

  #[inline]
  fn accept(&self, _: usize) -> bool {
    true
  }
}

impl Repeat for RangeInclusive<usize> {
  #[inline]
  fn validate(&self, repeated: usize) -> bool {
    repeated < *self.end()
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeTo<usize> {
  #[inline]
  fn validate(&self, repeated: usize) -> bool {
    repeated + 1 < self.end
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeToInclusive<usize> {
  #[inline]
  fn validate(&self, repeated: usize) -> bool {
    repeated < self.end
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

/// A [`Parse`] implementor created by `*`.
#[derive(Debug, Clone, Copy)]
pub struct Mul<Lhs, Rhs> {
  pub lhs: Lhs,
  pub rhs: Rhs,
}

impl<Lhs, Rhs> Mul<Lhs, Rhs> {
  #[inline]
  pub fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self { lhs, rhs }
  }
}

#[inline]
fn impl_mul<'text, Kind, State, Heap, Acc>(
  lhs: &impl Parse<State, Heap, Kind = Kind>,
  range: &impl Repeat,
  init: impl Fn() -> Acc,
  folder: impl Fn(Kind, Acc) -> Acc,
  input: &mut Input<'text, &mut State, &mut Heap>,
) -> Option<Output<'text, Acc>> {
  let mut repeated = 0;
  let mut output = Output {
    kind: init(),
    rest: input.rest(),
  };

  while range.validate(repeated) {
    let Some(mut input) = input.reload(output.rest) else {
      break;
    };
    let Some(next_output) = lhs.parse(&mut input) else {
      break;
    };
    output.rest = next_output.rest;
    output.kind = folder(next_output.kind, output.kind);
    repeated += 1;
  }

  range.accept(repeated).then_some(output)
}

impl<
    State,
    Heap,
    Lhs: Parse<State, Heap>,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Kind, Acc) -> Acc,
  > Parse<State, Heap> for Mul<Lhs, (Repeater, Initializer, InlineFolder)>
{
  type Kind = Acc;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Acc>> {
    let (range, init, folder) = &self.rhs;
    impl_mul(&self.lhs, range, init, folder, input)
  }
}

impl<
    State,
    Heap,
    Lhs: Parse<State, Heap>,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Kind, Acc) -> Acc,
  > ops::Mul<(Repeater, Initializer, InlineFolder)> for Combinator<State, Heap, Lhs>
{
  type Output = Combinator<State, Heap, Mul<Lhs, (Repeater, Initializer, InlineFolder)>>;

  /// Create a new combinator to repeat the original combinator
  /// with the given repetition range, accumulator initializer and folder.
  /// The combinator will return the output with the [`Fold`]-ed kind value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: (Repeater, Initializer, InlineFolder)) -> Self::Output {
    Self::Output::new(Mul::new(self.parser, rhs))
  }
}

/// A helper trait to accumulate kind values when performing `*` on [`Combinator`]s.
///
/// Built-in implementations are provided for `()`.
/// # Examples
/// ## Inline Fold
/// For simple cases, you can accumulate the kind values inline, without using this trait.
/// ```
/// # use whitehole::{combinator::next, parse::Input};
/// let combinator =
///   // accept one ascii digit at a time
///   next(|c| c.is_ascii_digit())
///     // convert the char to a number
///     .select(|ctx| ctx.input.next() as usize - '0' as usize)
///     // repeat for 1 or more times, init accumulator with 0, and fold kind values
///     * (1.., || 0, |kind, acc| acc * 10 + kind);
///
/// // parse "123" to 123
/// assert_eq!(
///   combinator.parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()).unwrap().kind,
///   123
/// )
/// ```
/// ## Fold with Custom Type
/// If you want to re-use the folder logic, you can implement this trait for a custom type.
/// ```
/// # use whitehole::{combinator::{operator::mul::Fold, next}, parse::Input};
/// // since you can't implement `Fold` for `usize` directly,
/// // wrap it in a new-type
/// struct Usize(usize);
///
/// impl Fold for Usize {
///   type Output = usize;
///
///   fn fold(self, acc: Self::Output) -> Self::Output {
///     acc * 10 + self.0
///   }
/// }
///
/// let combinator =
///   // accept one ascii digit at a time
///   next(|c| c.is_ascii_digit())
///     // convert the char to a number, wrapped in `Usize`
///     .select(|ctx| Usize(ctx.input.next() as usize - '0' as usize))
///     // repeat for 1 or more times, fold `Usize` to `usize`
///     * (1..);
///     // equals to: `* (1.., Usize::Output::default, Usize::fold)`
///
/// // parse "123" to 123
/// assert_eq!(
///   combinator.parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()).unwrap().kind,
///   123
/// )
/// ```
pub trait Fold {
  /// The accumulator type.
  type Output: Default;

  /// Fold self with the accumulator.
  fn fold(self, acc: Self::Output) -> Self::Output;
}

impl Fold for () {
  type Output = ();
  #[inline]
  fn fold(self, _: Self::Output) -> Self::Output {}
}

impl<State, Heap, Lhs: Parse<State, Heap, Kind: Fold>, Rhs: Repeat> ops::Mul<Rhs>
  for Combinator<State, Heap, Lhs>
{
  type Output = Combinator<State, Heap, Mul<Lhs, Rhs>>;

  /// Create a new combinator to repeat the original combinator for `rhs` times.
  /// The combinator will return the output with the [`Fold`]-ed kind value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul::new(self.parser, rhs))
  }
}

impl<State, Heap, Lhs: Parse<State, Heap, Kind: Fold>, Rhs: Repeat> Parse<State, Heap>
  for Mul<Lhs, Rhs>
{
  type Kind = <Lhs::Kind as Fold>::Output;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    impl_mul(
      &self.lhs,
      &self.rhs,
      Self::Kind::default,
      Lhs::Kind::fold,
      input,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::{wrap, Input, Output};

  #[derive(Debug)]
  struct MyKind(usize);
  impl Fold for MyKind {
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
          kind: MyKind(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * 3)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    let n = 0;
    assert_eq!(
      (rejecter() * n).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    let n = 0;
    assert_eq!(
      (accepter() * n).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        rest: "123",
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * 3).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { kind: 3, rest: "" })
    );

    // overflow, reject
    assert!((accepter() * 4)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..2))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..1)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        rest: "123",
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (0..3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { kind: 1, rest: "3" })
    );

    // too few, reject
    assert!((accepter() * (4..6))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_from() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "123",
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (0..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { kind: 3, rest: "" })
    );

    // too few, reject
    assert!((accepter() * (4..))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_full() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "123",
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { kind: 3, rest: "" })
    );
  }

  #[test]
  fn combinator_mul_range_inclusive() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..=3))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..=2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..=0)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        rest: "123",
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (0..=3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { kind: 3, rest: "" })
    );

    // too few, reject
    assert!((accepter() * (4..=6))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_to() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..1)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        rest: "123",
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (..3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { kind: 1, rest: "3" })
    );
  }

  #[test]
  fn combinator_mul_range_to_inclusive() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..=2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..=0)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        rest: "123",
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (..=3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { kind: 3, rest: "" })
    );
  }
}
