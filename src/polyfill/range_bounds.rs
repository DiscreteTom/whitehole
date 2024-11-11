use std::ops::{Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/// A polyfill for [`std::ops::RangeBounds`]
/// because [`usize`] is not [`std::ops::RangeBounds<usize>`] yet,
/// but in the future it might be.
///
/// Built-in implementations are provided for [`usize`], [`Range`], [`RangeFrom`],
/// [`RangeFull`], [`RangeInclusive`], [`RangeTo`], and [`RangeToInclusive`].
///
/// See [`std::ops::RangeBounds`].
pub trait RangeBounds<T: ?Sized> {
  /// See [`std::ops::RangeBounds::start_bound`].
  fn start_bound(&self) -> Bound<&T>;
  /// See [`std::ops::RangeBounds::end_bound`].
  fn end_bound(&self) -> Bound<&T>;
  /// See [`std::ops::RangeBounds::contains`].
  fn contains<U>(&self, item: &U) -> bool
  where
    T: PartialOrd<U>,
    U: ?Sized + PartialOrd<T>,
  {
    // copied from std::ops::RangeBounds::contains
    // TODO: is there a better way to prevent copying the code?
    (match self.start_bound() {
      Bound::Included(start) => start <= item,
      Bound::Excluded(start) => start < item,
      Bound::Unbounded => true,
    }) && (match self.end_bound() {
      Bound::Included(end) => item <= end,
      Bound::Excluded(end) => item < end,
      Bound::Unbounded => true,
    })
  }
}

impl RangeBounds<usize> for usize {
  fn start_bound(&self) -> Bound<&usize> {
    Bound::Included(self)
  }
  fn end_bound(&self) -> Bound<&usize> {
    Bound::Included(self)
  }
}

macro_rules! impl_range_bounds_for_range {
  ($range:ty) => {
    impl<T> RangeBounds<T> for $range {
      fn start_bound(&self) -> Bound<&T> {
        std::ops::RangeBounds::start_bound(self)
      }
      fn end_bound(&self) -> Bound<&T> {
        std::ops::RangeBounds::end_bound(self)
      }
    }
  };
}
impl_range_bounds_for_range!(Range<T>);
impl_range_bounds_for_range!(RangeFrom<T>);
impl_range_bounds_for_range!(RangeFull);
impl_range_bounds_for_range!(RangeInclusive<T>);
impl_range_bounds_for_range!(RangeTo<T>);
impl_range_bounds_for_range!(RangeToInclusive<T>);

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn polyfill_range_bounds() {
    fn in_range(r: impl RangeBounds<usize>, n: usize) -> bool {
      r.contains(&n)
    }

    // usize
    assert!(!in_range(0, 1));
    assert!(in_range(1, 1));
    assert!(!in_range(2, 1));

    // Range
    assert!(!in_range(0..1, 1));
    assert!(in_range(0..2, 1));
    assert!(in_range(1..2, 1));

    // RangeFrom
    assert!(in_range(0.., 1));
    assert!(in_range(1.., 1));
    assert!(!in_range(2.., 1));

    // RangeFull
    assert!(in_range(.., 1));

    // RangeInclusive
    assert!(!in_range(0..=0, 1));
    assert!(in_range(0..=1, 1));
    assert!(in_range(0..=2, 1));
    assert!(in_range(1..=1, 1));
    assert!(in_range(1..=2, 1));
    assert!(!in_range(2..=2, 1));

    // RangeTo
    assert!(!in_range(..0, 1));
    assert!(!in_range(..1, 1));
    assert!(in_range(..2, 1));

    // RangeToInclusive
    assert!(!in_range(..=0, 1));
    assert!(in_range(..=1, 1));
    assert!(in_range(..=2, 1));
  }
}
