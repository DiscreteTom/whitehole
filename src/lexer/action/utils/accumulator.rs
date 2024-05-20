/// Accumulate values and emit a result.
pub trait Accumulator<T>: Clone {
  type Target: Default;

  /// Update the accumulator with a value.
  fn update(&mut self, t: &T);
  /// Consume the accumulator and emit the result.
  fn emit(self) -> Self::Target;
}

/// A mock [`Accumulator`] that does nothing.
/// This should only be used in `Option<Accumulator>` to represent [`None`].
/// # Panics
/// Panics if any [`Accumulator`] method is called.
#[derive(Clone, Debug)]
pub struct MockAccumulator;
impl<T> Accumulator<T> for MockAccumulator {
  type Target = ();
  fn update(&mut self, _: &T) {
    unreachable!("MockAccumulator::update should never be called")
  }
  fn emit(self) -> Self::Target {
    unreachable!("MockAccumulator::emit should never be called")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[should_panic]
  #[test]
  fn mock_accumulator_update() {
    let mut acc = MockAccumulator;
    acc.update(&'a');
  }

  #[should_panic]
  #[test]
  fn mock_accumulator_emit() {
    let acc = MockAccumulator;
    Accumulator::<()>::emit(acc);
  }
}
