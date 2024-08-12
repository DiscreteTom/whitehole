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

  /// Set the accumulator to `()` (discard all values).
  #[inline]
  pub fn to_mock<R>(self) -> R
  where
    C: FnOnce(()) -> R,
  {
    self.to(())
  }

  /// Accumulate values into a new vector.
  /// # Performance
  /// Creating a new vector each time is inefficient.
  /// This should only be used for fast prototyping.
  ///
  /// In production if you want to collect values into a vector,
  /// to prevent unnecessary allocations and de-allocations,
  /// you should provision the capacity using [`Vec::with_capacity`]
  /// and re-use the vector as much as possible with [`Self::to`].
  #[inline]
  pub fn to_vec<V, R>(self) -> R
  where
    C: FnOnce(Vec<V>) -> R,
  {
    self.to(vec![])
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
