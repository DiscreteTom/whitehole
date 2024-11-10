use crate::combinator::{Combinator, Output};
use std::ops::{
  Bound, Mul, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

/// A helper trait to fold kind values when calling [`Mul`] on [`Combinator`].
///
/// Built-in implementations are provided for `()`.
pub trait Fold {
  /// The fold result type.
  type Output: Default;
  /// Fold self with the current value.
  fn fold(self, current: Self::Output) -> Self::Output;
}

impl Fold for () {
  type Output = ();
  fn fold(self, _: Self::Output) -> Self::Output {}
}

// TODO: Currently `usize` is not `RangeBounds<usize>` but in the future it might be.
// At that time this mod could be simplified with
// `impl Mul<RangeBounds> for Combinator`

fn impl_mul_for_range_bound<'a, Kind: Fold + 'a, State: 'a, Heap: 'a>(
  lhs: Combinator<'a, Kind, State, Heap>,
  rhs: impl RangeBounds<usize> + 'a,
) -> Combinator<'a, Kind::Output, State, Heap> {
  Combinator::boxed(move |input| {
    // if repeat at most 0 times, just return the default value
    if match rhs.end_bound() {
      Bound::Included(&end) => end == 0,
      Bound::Excluded(&end) => end <= 1,
      Bound::Unbounded => false,
    } {
      return Some(Output {
        kind: Kind::Output::default(),
        digested: 0,
      });
    }

    let (repeated, output) = match lhs.parse(input) {
      None => {
        // the first parse is rejected,
        // accept if 0 is included in the range
        if match rhs.start_bound() {
          Bound::Included(&start) => start == 0,
          Bound::Excluded(_) => false, // usize cannot be negative
          Bound::Unbounded => true,
        } {
          return Some(Output {
            kind: Kind::Output::default(),
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
        let mut output = output.map(|kind| kind.fold(Kind::Output::default()));
        while match rhs.end_bound() {
          Bound::Included(&end) => repeated < end,
          Bound::Excluded(&end) => repeated + 1 < end,
          Bound::Unbounded => true,
        } {
          match input
            .digest(output.digested)
            .and_then(|mut input| lhs.parse(&mut input))
          {
            Some(next_output) => {
              output.digested += next_output.digested;
              output.kind = next_output.kind.fold(output.kind);
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
    match rhs.start_bound() {
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

impl<'a, Kind: Fold + 'a, State: 'a, Heap: 'a> Mul<usize> for Combinator<'a, Kind, State, Heap> {
  type Output = Combinator<'a, Kind::Output, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the [`Fold`]-ed kind value and the sum of the digested.
  fn mul(self, rhs: usize) -> Self::Output {
    impl_mul_for_range_bound(self, rhs..=rhs)
  }
}

impl<'a, Kind: Fold + 'a, State: 'a, Heap: 'a> Mul<Range<usize>>
  for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, Kind::Output, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the [`Fold`]-ed kind value and the sum of the digested.
  fn mul(self, rhs: Range<usize>) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
  }
}

impl<'a, Kind: Fold + 'a, State: 'a, Heap: 'a> Mul<RangeFrom<usize>>
  for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, Kind::Output, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the [`Fold`]-ed kind value and the sum of the digested.
  fn mul(self, rhs: RangeFrom<usize>) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
  }
}

impl<'a, Kind: Fold + 'a, State: 'a, Heap: 'a> Mul<RangeFull>
  for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, Kind::Output, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the [`Fold`]-ed kind value and the sum of the digested.
  fn mul(self, rhs: RangeFull) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
  }
}

impl<'a, Kind: Fold + 'a, State: 'a, Heap: 'a> Mul<RangeInclusive<usize>>
  for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, Kind::Output, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the [`Fold`]-ed kind value and the sum of the digested.
  fn mul(self, rhs: RangeInclusive<usize>) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
  }
}

impl<'a, Kind: Fold + 'a, State: 'a, Heap: 'a> Mul<RangeTo<usize>>
  for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, Kind::Output, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the [`Fold`]-ed kind value and the sum of the digested.
  fn mul(self, rhs: RangeTo<usize>) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
  }
}

impl<'a, Kind: Fold + 'a, State: 'a, Heap: 'a> Mul<RangeToInclusive<usize>>
  for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, Kind::Output, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the [`Fold`]-ed kind value and the sum of the digested.
  fn mul(self, rhs: RangeToInclusive<usize>) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
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
