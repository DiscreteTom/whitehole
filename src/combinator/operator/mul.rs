//! Overload [`Mul`] operator for [`Combinator`].

use crate::{
  combinator::{Combinator, Output},
  polyfill::RangeBounds,
};
use std::ops::{Bound, Mul};

impl<
    'a,
    Kind: 'a,
    State: 'a,
    Heap: 'a,
    Range: RangeBounds<usize> + 'a,
    Acc,
    Initializer: Fn() -> Acc + 'a,
    InlineFolder: Fn(Kind, Acc) -> Acc + 'a,
  > Mul<(Range, Initializer, InlineFolder)> for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, Acc, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the [`Fold`]-ed kind value and the sum of the digested.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: (Range, Initializer, InlineFolder)) -> Self::Output {
    let (range, init, folder) = rhs;
    Combinator::boxed(move |input| {
      // if repeat at most 0 times, just return the default value
      if match range.end_bound() {
        Bound::Included(&end) => end == 0,
        Bound::Excluded(&end) => end <= 1,
        Bound::Unbounded => false,
      } {
        return Some(Output {
          kind: init(),
          digested: 0,
        });
      }

      let (repeated, output) = match self.parse(input) {
        None => {
          // the first parse is rejected,
          // accept if 0 is included in the range
          if match range.start_bound() {
            Bound::Included(&start) => start == 0,
            Bound::Excluded(_) => false, // usize cannot be negative
            Bound::Unbounded => true,
          } {
            return Some(Output {
              kind: init(),
              digested: 0,
            });
          }
          // else, repeat 0 times is not allowed, reject
          return None;
        }
        Some(output) => {
          let mut repeated = 1;
          // generate the target kind value here instead of outer scope
          // to prevent unnecessary creation of the default value
          let mut output = output.map(|kind| folder(kind, init()));
          while match range.end_bound() {
            Bound::Included(&end) => repeated < end,
            Bound::Excluded(&end) => repeated + 1 < end,
            Bound::Unbounded => true,
          } {
            match input
              .digest(output.digested)
              .and_then(|mut input| self.parse(&mut input))
            {
              Some(next_output) => {
                output.digested += next_output.digested;
                output.kind = folder(next_output.kind, output.kind);
                repeated += 1;
              }
              None => {
                // end of input, or rejected
                // proceed with current output
                break;
              }
            }
          }
          (repeated, output)
        }
      };

      // reject if repeated times is too few
      match range.start_bound() {
        Bound::Included(&start) => {
          if repeated < start {
            return None;
          }
        }
        Bound::Excluded(&start) => {
          if repeated <= start {
            return None;
          }
        }
        Bound::Unbounded => {}
      }

      output.into()
    })
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
  fn fold(self, _: Self::Output) -> Self::Output {}
}

impl<'a, Kind: Fold + 'a, State: 'a, Heap: 'a, Range: RangeBounds<usize> + 'a> Mul<Range>
  for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, Kind::Output, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the [`Fold`]-ed kind value and the sum of the digested.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: Range) -> Self::Output {
    self * (rhs, Kind::Output::default, Kind::fold)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::{Input, Output};

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
    let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
    let accepter = || {
      Combinator::boxed(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          digested: 1,
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
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    let n = 0;
    assert_eq!(
      (accepter() * n).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        digested: 0,
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * 3).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 3,
        digested: 3,
      })
    );

    // overflow, reject
    assert!((accepter() * 4)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range() {
    let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
    let accepter = || {
      Combinator::boxed(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          digested: 1,
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
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..1)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        digested: 0,
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (0..3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 1,
        digested: 2,
      })
    );

    // too few, reject
    assert!((accepter() * (4..6))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_from() {
    let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
    let accepter = || {
      Combinator::boxed(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          digested: 1,
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
        digested: 0,
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (0..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 3,
        digested: 3,
      })
    );

    // too few, reject
    assert!((accepter() * (4..))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_full() {
    let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
    let accepter = || {
      Combinator::boxed(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          digested: 1,
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 0,
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 3,
        digested: 3,
      })
    );
  }

  #[test]
  fn combinator_mul_range_inclusive() {
    let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
    let accepter = || {
      Combinator::boxed(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          digested: 1,
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
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..=0)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        digested: 0,
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (0..=3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 3,
        digested: 3,
      })
    );

    // too few, reject
    assert!((accepter() * (4..=6))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_to() {
    let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
    let accepter = || {
      Combinator::boxed(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          digested: 1,
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..1)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        digested: 0,
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (..3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 1,
        digested: 2,
      })
    );
  }

  #[test]
  fn combinator_mul_range_to_inclusive() {
    let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
    let accepter = || {
      Combinator::boxed(|input| {
        Some(Output {
          kind: MyKind(input.start()),
          digested: 1,
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..=2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 0,
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..=0)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        digested: 0,
      })
    );

    // normal, apply the folded kind value and sum the digested
    assert_eq!(
      (accepter() * (..=3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 3,
        digested: 3,
      })
    );
  }
}
