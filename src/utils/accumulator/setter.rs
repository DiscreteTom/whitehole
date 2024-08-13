use super::DebugAccumulator;

pub struct AccumulatorSetter<C> {
  closure: C,
}

impl<C> AccumulatorSetter<C> {
  #[inline]
  pub const fn new(closure: C) -> Self {
    Self { closure }
  }
}

impl<C> AccumulatorSetter<C> {
  /// Set a custom accumulator.
  #[inline]
  pub fn to<R, Acc>(self, acc: Acc) -> R
  where
    C: FnOnce(Acc) -> R,
  {
    (self.closure)(acc)
  }

  /// Print the values to stdout (for debugging).
  ///
  /// Since [`DebugAccumulator`] is not a container,
  /// no allocation will be made.
  #[inline]
  pub fn to_stdout<R>(self) -> R
  where
    C: FnOnce(DebugAccumulator) -> R,
  {
    self.to(DebugAccumulator)
  }
}
