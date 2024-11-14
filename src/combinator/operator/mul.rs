//! Overload [`Mul`] operator for combinator.

use crate::{
  combinator::{Input, Output, Parse},
  impl_combinator,
};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

// TODO: better name
pub trait Repeat {
  // TODO: better name
  fn should_repeat(&self, repeat: usize) -> bool;
  fn should_accept(&self, repeat: usize) -> bool;
}

impl Repeat for usize {
  #[inline]
  fn should_repeat(&self, repeat: usize) -> bool {
    repeat < *self
  }

  #[inline]
  fn should_accept(&self, repeat: usize) -> bool {
    repeat == *self
  }
}

impl Repeat for Range<usize> {
  #[inline]
  fn should_repeat(&self, repeat: usize) -> bool {
    repeat + 1 < self.end
  }

  #[inline]
  fn should_accept(&self, repeat: usize) -> bool {
    self.contains(&repeat)
  }
}
impl Repeat for RangeFrom<usize> {
  #[inline]
  fn should_repeat(&self, _: usize) -> bool {
    true
  }

  #[inline]
  fn should_accept(&self, repeat: usize) -> bool {
    self.contains(&repeat)
  }
}
impl Repeat for RangeFull {
  #[inline]
  fn should_repeat(&self, _: usize) -> bool {
    true
  }

  #[inline]
  fn should_accept(&self, _: usize) -> bool {
    true
  }
}
impl Repeat for RangeInclusive<usize> {
  #[inline]
  fn should_repeat(&self, repeat: usize) -> bool {
    repeat < *self.end()
  }

  #[inline]
  fn should_accept(&self, repeat: usize) -> bool {
    self.contains(&repeat)
  }
}
impl Repeat for RangeTo<usize> {
  #[inline]
  fn should_repeat(&self, repeat: usize) -> bool {
    repeat + 1 < self.end
  }

  #[inline]
  fn should_accept(&self, repeat: usize) -> bool {
    self.contains(&repeat)
  }
}
impl Repeat for RangeToInclusive<usize> {
  #[inline]
  fn should_repeat(&self, repeat: usize) -> bool {
    repeat < self.end
  }

  #[inline]
  fn should_accept(&self, repeat: usize) -> bool {
    self.contains(&repeat)
  }
}

/// A composite combinator created by `*`.
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

impl_combinator!(Mul<Lhs, Rhs>, Lhs, Rhs);

impl<
    State,
    Heap,
    Lhs: Parse<State, Heap>,
    Acc,
    Range: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Kind, Acc) -> Acc,
  > Parse<State, Heap> for Mul<Lhs, (Range, Initializer, InlineFolder)>
{
  type Kind = Acc;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    let (range, init, folder) = &self.rhs;
    let mut repeated = 0;
    let mut output = Output {
      kind: init(),
      rest: input.rest(),
    };
    while range.should_repeat(repeated) {
      let Some(mut input) = input.reload(output.rest) else {
        break;
      };
      let Some(next_output) = self.lhs.parse(&mut input) else {
        break;
      };
      output.rest = next_output.rest;
      output.kind = folder(next_output.kind, output.kind);
      repeated += 1;
    }

    // reject if repeated times is too few
    range.should_accept(repeated).then_some(output)
  }
}

/// A helper trait to accumulate kind values when calling [`Mul`] on [`Combinator`].
///
/// Built-in implementations are provided for `()`.
/// # Examples
/// ## Inline Fold
/// For simple cases, you can accumulate the kind values inline, without using this trait.
/// ```
/// # use whitehole::combinator::{Combinator, next, Input};
/// let combinator: Combinator<usize> =
///   // accept one ascii digit at a time
///   next(|c| c.is_ascii_digit())
///     // convert the char to a number
///     .select(|ctx| ctx.input.next() as usize - '0' as usize)
///     // repeat 1 or more times, init accumulator with 0, and fold kind values
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
/// # use whitehole::combinator::{operator::mul::Fold, Combinator, next, Input};
/// // since you can't implement `Fold` for `usize` directly,
/// // wrap it in a new-type
/// struct DecimalNumber(usize);
///
/// impl Fold for DecimalNumber {
///   type Output = usize;
///   fn fold(self, acc: Self::Output) -> Self::Output {
///     acc * 10 + self.0
///   }
/// }
///
/// let combinator: Combinator<usize> =
///   // accept one ascii digit at a time
///   next(|c| c.is_ascii_digit())
///     // convert the char to a number, wrapped in `DecimalNumber`
///     .select(|ctx| DecimalNumber(ctx.input.next() as usize - '0' as usize))
///     // repeat 1 or more times, fold `DecimalNumber` to `usize`
///     * (1..);
///     // equals to: `* (1.., usize::default, DecimalNumber::fold)`
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

impl<State, Heap, Lhs: Parse<State, Heap, Kind: Fold<Output: Default>>, Rhs: Repeat>
  Parse<State, Heap> for Mul<Lhs, Rhs>
{
  type Kind = <Lhs::Kind as Fold>::Output;

  // TODO: merge dup code
  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    let range = &self.rhs;
    let mut repeated = 0;
    let mut output = Output {
      kind: Default::default(),
      rest: input.rest(),
    };
    while range.should_repeat(repeated) {
      let Some(mut input) = input.reload(output.rest) else {
        break;
      };
      let Some(next_output) = self.lhs.parse(&mut input) else {
        break;
      };
      output.rest = next_output.rest;
      output.kind = next_output.kind.fold(output.kind);
      repeated += 1;
    }

    // reject if repeated times is too few
    range.should_accept(repeated).then_some(output)
  }
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use crate::combinator::{Input, Output};

//   #[derive(Debug)]
//   struct MyKind(usize);
//   impl Fold for MyKind {
//     type Output = usize;
//     fn fold(self, current: Self::Output) -> Self::Output {
//       self.0 + current
//     }
//   }

//   #[test]
//   fn combinator_mul_usize() {
//     let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
//     let accepter = || {
//       Combinator::boxed(|input| {
//         Some(Output {
//           kind: MyKind(input.start()),
//           rest: &input.rest()[1..],
//         })
//       })
//     };

//     // repeat a rejecter will reject
//     assert!((rejecter() * 3)
//       .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//       .is_none());

//     // repeat rejecter 0 times will accept
//     let n = 0;
//     assert_eq!(
//       (rejecter() * n).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "123",
//       })
//     );

//     // repeat an accepter 0 times will accept
//     let n = 0;
//     assert_eq!(
//       (accepter() * n).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: 0,
//         rest: "123",
//       })
//     );

//     // normal, apply the folded kind value and sum the digested
//     assert_eq!(
//       (accepter() * 3).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output { kind: 3, rest: "" })
//     );

//     // overflow, reject
//     assert!((accepter() * 4)
//       .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//       .is_none());
//   }

//   #[test]
//   fn combinator_mul_range() {
//     let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
//     let accepter = || {
//       Combinator::boxed(|input| {
//         Some(Output {
//           kind: MyKind(input.start()),
//           rest: &input.rest()[1..],
//         })
//       })
//     };

//     // repeat a rejecter will reject
//     assert!((rejecter() * (1..2))
//       .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//       .is_none());

//     // repeat rejecter 0 times will accept
//     assert_eq!(
//       (rejecter() * (0..2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "123",
//       })
//     );

//     // repeat an accepter 0 times will accept
//     assert_eq!(
//       (accepter() * (0..1)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: 0,
//         rest: "123",
//       })
//     );

//     // normal, apply the folded kind value and sum the digested
//     assert_eq!(
//       (accepter() * (0..3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output { kind: 1, rest: "3" })
//     );

//     // too few, reject
//     assert!((accepter() * (4..6))
//       .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//       .is_none());
//   }

//   #[test]
//   fn combinator_mul_range_from() {
//     let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
//     let accepter = || {
//       Combinator::boxed(|input| {
//         Some(Output {
//           kind: MyKind(input.start()),
//           rest: &input.rest()[1..],
//         })
//       })
//     };

//     // repeat a rejecter will reject
//     assert!((rejecter() * (1..))
//       .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//       .is_none());

//     // repeat rejecter 0 times will accept
//     assert_eq!(
//       (rejecter() * (0..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "123",
//       })
//     );

//     // normal, apply the folded kind value and sum the digested
//     assert_eq!(
//       (accepter() * (0..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output { kind: 3, rest: "" })
//     );

//     // too few, reject
//     assert!((accepter() * (4..))
//       .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//       .is_none());
//   }

//   #[test]
//   fn combinator_mul_range_full() {
//     let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
//     let accepter = || {
//       Combinator::boxed(|input| {
//         Some(Output {
//           kind: MyKind(input.start()),
//           rest: &input.rest()[1..],
//         })
//       })
//     };

//     // repeat rejecter 0 times will accept
//     assert_eq!(
//       (rejecter() * (..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "123",
//       })
//     );

//     // normal, apply the folded kind value and sum the digested
//     assert_eq!(
//       (accepter() * (..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output { kind: 3, rest: "" })
//     );
//   }

//   #[test]
//   fn combinator_mul_range_inclusive() {
//     let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
//     let accepter = || {
//       Combinator::boxed(|input| {
//         Some(Output {
//           kind: MyKind(input.start()),
//           rest: &input.rest()[1..],
//         })
//       })
//     };

//     // repeat a rejecter will reject
//     assert!((rejecter() * (1..=3))
//       .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//       .is_none());

//     // repeat rejecter 0 times will accept
//     assert_eq!(
//       (rejecter() * (0..=2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "123",
//       })
//     );

//     // repeat an accepter 0 times will accept
//     assert_eq!(
//       (accepter() * (0..=0)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: 0,
//         rest: "123",
//       })
//     );

//     // normal, apply the folded kind value and sum the digested
//     assert_eq!(
//       (accepter() * (0..=3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output { kind: 3, rest: "" })
//     );

//     // too few, reject
//     assert!((accepter() * (4..=6))
//       .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
//       .is_none());
//   }

//   #[test]
//   fn combinator_mul_range_to() {
//     let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
//     let accepter = || {
//       Combinator::boxed(|input| {
//         Some(Output {
//           kind: MyKind(input.start()),
//           rest: &input.rest()[1..],
//         })
//       })
//     };

//     // repeat rejecter 0 times will accept
//     assert_eq!(
//       (rejecter() * (..2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "123",
//       })
//     );

//     // repeat an accepter 0 times will accept
//     assert_eq!(
//       (accepter() * (..1)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: 0,
//         rest: "123",
//       })
//     );

//     // normal, apply the folded kind value and sum the digested
//     assert_eq!(
//       (accepter() * (..3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output { kind: 1, rest: "3" })
//     );
//   }

//   #[test]
//   fn combinator_mul_range_to_inclusive() {
//     let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
//     let accepter = || {
//       Combinator::boxed(|input| {
//         Some(Output {
//           kind: MyKind(input.start()),
//           rest: &input.rest()[1..],
//         })
//       })
//     };

//     // repeat rejecter 0 times will accept
//     assert_eq!(
//       (rejecter() * (..=2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "123",
//       })
//     );

//     // repeat an accepter 0 times will accept
//     assert_eq!(
//       (accepter() * (..=0)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output {
//         kind: 0,
//         rest: "123",
//       })
//     );

//     // normal, apply the folded kind value and sum the digested
//     assert_eq!(
//       (accepter() * (..=3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
//       Some(Output { kind: 3, rest: "" })
//     );
//   }
// }
