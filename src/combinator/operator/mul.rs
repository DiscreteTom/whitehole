use crate::combinator::Combinator;
use std::ops::{
  Bound, Mul, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

// TODO: Currently `usize` is not `RangeBounds<usize>` but in the future it might be.
// At that time this mod could be simplified with
// `impl Mul<RangeBounds> for Combinator`

fn impl_mul_for_range_bound<'a, Kind: 'a, State: 'a, Heap: 'a>(
  lhs: Combinator<'a, Kind, State, Heap>,
  rhs: impl RangeBounds<usize> + 'a,
) -> Combinator<'a, Kind, State, Heap> {
  Combinator::boxed(move |input| {
    // reject if repeat 0 times
    match rhs.end_bound() {
      Bound::Included(&end) => {
        if end == 0 {
          return None;
        }
      }
      Bound::Excluded(&end) => {
        if end <= 1 {
          return None;
        }
      }
      Bound::Unbounded => {}
    }

    // now the combinator is expected to repeat at least once

    let mut repeated = 0;
    let output = lhs.parse(input).and_then(|mut output| {
      repeated += 1;
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
            output.kind = next_output.kind;
            repeated += 1;
          }
          None => {
            // end of input, or rejected
            // proceed with current output
            break;
          }
        }
      }
      output.into()
    })?;

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

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Mul<usize> for Combinator<'a, Kind, State, Heap> {
  type Output = Combinator<'a, Kind, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the kind of the last output and the sum of the digested.
  fn mul(self, rhs: usize) -> Self::Output {
    impl_mul_for_range_bound(self, rhs..=rhs)
  }
}

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Mul<Range<usize>> for Combinator<'a, Kind, State, Heap> {
  type Output = Combinator<'a, Kind, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the kind of the last output and the sum of the digested.
  fn mul(self, rhs: Range<usize>) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
  }
}

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Mul<RangeFrom<usize>>
  for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, Kind, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the kind of the last output and the sum of the digested.
  fn mul(self, rhs: RangeFrom<usize>) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
  }
}

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Mul<RangeFull> for Combinator<'a, Kind, State, Heap> {
  type Output = Combinator<'a, Kind, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the kind of the last output and the sum of the digested.
  fn mul(self, rhs: RangeFull) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
  }
}

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Mul<RangeInclusive<usize>>
  for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, Kind, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the kind of the last output and the sum of the digested.
  fn mul(self, rhs: RangeInclusive<usize>) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
  }
}

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Mul<RangeTo<usize>> for Combinator<'a, Kind, State, Heap> {
  type Output = Combinator<'a, Kind, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the kind of the last output and the sum of the digested.
  fn mul(self, rhs: RangeTo<usize>) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
  }
}

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Mul<RangeToInclusive<usize>>
  for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, Kind, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the kind of the last output and the sum of the digested.
  fn mul(self, rhs: RangeToInclusive<usize>) -> Self::Output {
    impl_mul_for_range_bound(self, rhs)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::{Input, Output};

  #[test]
  fn combinator_mul() {
    let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
    let accepter = || {
      Combinator::boxed(|input| {
        Some(Output {
          kind: input.next(),
          digested: 1,
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * 3)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat an accepter 0 times will reject
    let n = 0;
    assert!((accepter() * n)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // normal, apply the last output's kind and sum the digested
    assert_eq!(
      (accepter() * 3).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: '3',
        digested: 3,
      })
    );

    // overflow, reject
    assert!((accepter() * 4)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }
}
