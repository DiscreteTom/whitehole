use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/// A helper trait to represent repetition when performing `*`
/// on [`Combinator`](crate::combinator::Combinator)s.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn repeat_usize() {
    assert_eq!(0.validate(0), false);
    assert_eq!(0.validate(1), false);
    assert_eq!(0.accept(0), true);
    assert_eq!(0.accept(1), false);

    assert_eq!(1.validate(0), true);
    assert_eq!(1.validate(1), false);
    assert_eq!(1.validate(2), false);
    assert_eq!(1.accept(0), false);
    assert_eq!(1.accept(1), true);
    assert_eq!(1.accept(2), false);
  }

  #[test]
  fn repeat_range() {
    assert_eq!((1..3).validate(0), true);
    assert_eq!((1..3).validate(1), true);
    assert_eq!((1..3).validate(2), false);
    assert_eq!((1..3).validate(3), false);
    assert_eq!((1..3).validate(4), false);
    assert_eq!((1..3).accept(0), false);
    assert_eq!((1..3).accept(1), true);
    assert_eq!((1..3).accept(2), true);
    assert_eq!((1..3).accept(3), false);
    assert_eq!((1..3).accept(4), false);
  }

  #[test]
  fn repeat_range_from() {
    assert_eq!((1..).validate(0), true);
    assert_eq!((1..).validate(1), true);
    assert_eq!((1..).validate(2), true);
    assert_eq!((1..).validate(3), true);
    assert_eq!((1..).validate(4), true);
    assert_eq!((1..).accept(0), false);
    assert_eq!((1..).accept(1), true);
    assert_eq!((1..).accept(2), true);
    assert_eq!((1..).accept(3), true);
    assert_eq!((1..).accept(4), true);
  }

  #[test]
  fn repeat_range_full() {
    assert_eq!((..).validate(0), true);
    assert_eq!((..).validate(1), true);
    assert_eq!((..).validate(2), true);
    assert_eq!((..).validate(3), true);
    assert_eq!((..).validate(4), true);
    assert_eq!((..).accept(0), true);
    assert_eq!((..).accept(1), true);
    assert_eq!((..).accept(2), true);
    assert_eq!((..).accept(3), true);
    assert_eq!((..).accept(4), true);
  }

  #[test]
  fn repeat_range_inclusive() {
    assert_eq!((1..=3).validate(0), true);
    assert_eq!((1..=3).validate(1), true);
    assert_eq!((1..=3).validate(2), true);
    assert_eq!((1..=3).validate(3), false);
    assert_eq!((1..=3).validate(4), false);
    assert_eq!((1..=3).accept(0), false);
    assert_eq!((1..=3).accept(1), true);
    assert_eq!((1..=3).accept(2), true);
    assert_eq!((1..=3).accept(3), true);
    assert_eq!((1..=3).accept(4), false);
  }

  #[test]
  fn repeat_range_to() {
    assert_eq!((..3).validate(0), true);
    assert_eq!((..3).validate(1), true);
    assert_eq!((..3).validate(2), false);
    assert_eq!((..3).validate(3), false);
    assert_eq!((..3).validate(4), false);
    assert_eq!((..3).accept(0), true);
    assert_eq!((..3).accept(1), true);
    assert_eq!((..3).accept(2), true);
    assert_eq!((..3).accept(3), false);
    assert_eq!((..3).accept(4), false);
  }

  #[test]
  fn repeat_range_to_inclusive() {
    assert_eq!((..=3).validate(0), true);
    assert_eq!((..=3).validate(1), true);
    assert_eq!((..=3).validate(2), true);
    assert_eq!((..=3).validate(3), false);
    assert_eq!((..=3).validate(4), false);
    assert_eq!((..=3).accept(0), true);
    assert_eq!((..=3).accept(1), true);
    assert_eq!((..=3).accept(2), true);
    assert_eq!((..=3).accept(3), true);
    assert_eq!((..=3).accept(4), false);
  }
}
