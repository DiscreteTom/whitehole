use super::DebugAccumulator;
use std::marker::PhantomData;

pub struct AccumulatorSetter<C, R, V, Acc> {
  closure: C,
  /// Record the return type of the closure.
  _r: PhantomData<R>,
  /// Record the value type to accumulate.
  _e: PhantomData<V>,
  /// Record the accumulator type.
  _a: PhantomData<Acc>,
}

impl<C, R, E, Acc> AccumulatorSetter<C, R, E, Acc> {
  #[inline]
  pub const fn new(closure: C) -> Self {
    Self {
      closure,
      _r: PhantomData,
      _e: PhantomData,
      _a: PhantomData,
    }
  }
}

impl<C: FnOnce(Acc) -> R, R, E, Acc> AccumulatorSetter<C, R, E, Acc> {
  /// Set a custom accumulator.
  #[inline]
  pub fn to(self, acc: Acc) -> R {
    (self.closure)(acc)
  }
}

impl<C: FnOnce(()) -> R, R, E> AccumulatorSetter<C, R, E, ()> {
  /// Set the accumulator to `()`
  /// (discard all values).
  #[inline]
  pub fn to_mock(self) -> R {
    self.to(())
  }
}

impl<C: FnOnce(Vec<E>) -> R, R, E> AccumulatorSetter<C, R, E, Vec<E>> {
  /// Accumulate values into a vector.
  /// # Caveats
  /// Creating a new vector each time may be inefficient.
  /// This should only be used for testing or debugging.
  #[inline]
  pub fn to_vec(self) -> R {
    self.to(vec![])
  }
}

impl<C: FnOnce(DebugAccumulator) -> R, R, E> AccumulatorSetter<C, R, E, DebugAccumulator> {
  /// Print the values to stdout.
  #[inline]
  pub fn to_stdout(self) -> R {
    self.to(DebugAccumulator)
  }
}
