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
  /// # Safety
  /// The caller should ensure the `repeated` is increased by 1 from `0`,
  /// and stop calling this with greater `repeated` if this returns `false`.
  /// This will be checked using [`debug_assert!`].
  unsafe fn validate(&self, repeated: usize) -> bool;

  /// Check if the repetition should be accepted
  /// based on the current repeated times.
  fn accept(&self, repeated: usize) -> bool;
}

impl Repeat for usize {
  #[inline]
  unsafe fn validate(&self, repeated: usize) -> bool {
    repeated < *self
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    repeated == *self
  }
}

impl Repeat for Range<usize> {
  #[inline]
  unsafe fn validate(&self, repeated: usize) -> bool {
    debug_assert!(self.end >= repeated);
    self.end.unchecked_sub(repeated) > 1
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeFrom<usize> {
  #[inline]
  unsafe fn validate(&self, _: usize) -> bool {
    true
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeFull {
  #[inline]
  unsafe fn validate(&self, _: usize) -> bool {
    true
  }

  #[inline]
  fn accept(&self, _: usize) -> bool {
    true
  }
}

impl Repeat for RangeInclusive<usize> {
  #[inline]
  unsafe fn validate(&self, repeated: usize) -> bool {
    repeated < *self.end()
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeTo<usize> {
  #[inline]
  unsafe fn validate(&self, repeated: usize) -> bool {
    debug_assert!(self.end >= repeated);
    self.end.unchecked_sub(repeated) > 1
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeToInclusive<usize> {
  #[inline]
  unsafe fn validate(&self, repeated: usize) -> bool {
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
    assert_eq!(unsafe { 0.validate(0) }, false);
    assert_eq!(unsafe { 0.validate(1) }, false);
    assert_eq!(0.accept(0), true);
    assert_eq!(0.accept(1), false);

    assert_eq!(unsafe { 1.validate(0) }, true);
    assert_eq!(unsafe { 1.validate(1) }, false);
    assert_eq!(unsafe { 1.validate(2) }, false);
    assert_eq!(1.accept(0), false);
    assert_eq!(1.accept(1), true);
    assert_eq!(1.accept(2), false);
  }

  #[test]
  fn repeat_range() {
    assert_eq!(unsafe { (1..3).validate(0) }, true);
    assert_eq!(unsafe { (1..3).validate(1) }, true);
    assert_eq!(unsafe { (1..3).validate(2) }, false);
    assert_eq!(unsafe { (1..3).validate(3) }, false);
    assert_eq!((1..3).accept(0), false);
    assert_eq!((1..3).accept(1), true);
    assert_eq!((1..3).accept(2), true);
    assert_eq!((1..3).accept(3), false);
    assert_eq!((1..3).accept(4), false);
  }

  #[test]
  #[should_panic]
  fn repeat_range_overflow() {
    unsafe { (1..3).validate(4) };
  }

  #[test]
  fn repeat_range_from() {
    assert_eq!(unsafe { (1..).validate(0) }, true);
    assert_eq!(unsafe { (1..).validate(1) }, true);
    assert_eq!(unsafe { (1..).validate(2) }, true);
    assert_eq!(unsafe { (1..).validate(3) }, true);
    assert_eq!(unsafe { (1..).validate(4) }, true);
    assert_eq!((1..).accept(0), false);
    assert_eq!((1..).accept(1), true);
    assert_eq!((1..).accept(2), true);
    assert_eq!((1..).accept(3), true);
    assert_eq!((1..).accept(4), true);
  }

  #[test]
  fn repeat_range_full() {
    assert_eq!(unsafe { (..).validate(0) }, true);
    assert_eq!(unsafe { (..).validate(1) }, true);
    assert_eq!(unsafe { (..).validate(2) }, true);
    assert_eq!(unsafe { (..).validate(3) }, true);
    assert_eq!(unsafe { (..).validate(4) }, true);
    assert_eq!((..).accept(0), true);
    assert_eq!((..).accept(1), true);
    assert_eq!((..).accept(2), true);
    assert_eq!((..).accept(3), true);
    assert_eq!((..).accept(4), true);
  }

  #[test]
  fn repeat_range_inclusive() {
    assert_eq!(unsafe { (1..=3).validate(0) }, true);
    assert_eq!(unsafe { (1..=3).validate(1) }, true);
    assert_eq!(unsafe { (1..=3).validate(2) }, true);
    assert_eq!(unsafe { (1..=3).validate(3) }, false);
    assert_eq!(unsafe { (1..=3).validate(4) }, false);
    assert_eq!((1..=3).accept(0), false);
    assert_eq!((1..=3).accept(1), true);
    assert_eq!((1..=3).accept(2), true);
    assert_eq!((1..=3).accept(3), true);
    assert_eq!((1..=3).accept(4), false);
  }

  #[test]
  fn repeat_range_to() {
    assert_eq!(unsafe { (..3).validate(0) }, true);
    assert_eq!(unsafe { (..3).validate(1) }, true);
    assert_eq!(unsafe { (..3).validate(2) }, false);
    assert_eq!(unsafe { (..3).validate(3) }, false);
    assert_eq!((..3).accept(0), true);
    assert_eq!((..3).accept(1), true);
    assert_eq!((..3).accept(2), true);
    assert_eq!((..3).accept(3), false);
    assert_eq!((..3).accept(4), false);
  }

  #[test]
  #[should_panic]
  fn repeat_range_to_overflow() {
    unsafe { (..3).validate(4) };
  }

  #[test]
  fn repeat_range_to_inclusive() {
    assert_eq!(unsafe { (..=3).validate(0) }, true);
    assert_eq!(unsafe { (..=3).validate(1) }, true);
    assert_eq!(unsafe { (..=3).validate(2) }, true);
    assert_eq!(unsafe { (..=3).validate(3) }, false);
    assert_eq!(unsafe { (..=3).validate(4) }, false);
    assert_eq!((..=3).accept(0), true);
    assert_eq!((..=3).accept(1), true);
    assert_eq!((..=3).accept(2), true);
    assert_eq!((..=3).accept(3), true);
    assert_eq!((..=3).accept(4), false);
  }
}
