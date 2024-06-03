/// Accumulate values and emit a result.
pub trait Accumulator<T>: Clone {
  type Target: Default;

  /// Update the accumulator with a value.
  fn update(&mut self, t: &T);
  /// Consume the accumulator and emit the result.
  fn emit(self) -> Self::Target;
}

/// A mock [`Accumulator`] that does nothing.
/// Useful if you don't need to accumulate anything.
#[derive(Clone, Debug, Default)]
pub struct MockAccumulator;
impl<T> Accumulator<T> for MockAccumulator {
  type Target = ();
  fn update(&mut self, _: &T) {}
  fn emit(self) -> Self::Target {}
}

/// Accumulate values into a [`Vec`] and emit the [`Vec`].
#[derive(Clone, Debug, Default)]
pub struct VecAccumulator(Vec<usize>);
impl Accumulator<usize> for VecAccumulator {
  type Target = Vec<usize>;
  fn update(&mut self, c: &usize) {
    self.0.push(*c);
  }
  fn emit(self) -> Self::Target {
    self.0
  }
}

/// Accumulate values into a [`String`] and emit the [`String`].
#[derive(Clone, Debug, Default)]
pub struct StringAccumulator(String);
impl Accumulator<char> for StringAccumulator {
  type Target = String;
  // TODO: batch update with a String instead of one char?
  fn update(&mut self, c: &char) {
    self.0.push(*c);
  }
  fn emit(self) -> Self::Target {
    self.0
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

  #[test]
  fn vec_accumulator() {
    let mut acc = VecAccumulator::default();
    acc.update(&1);
    acc.update(&2);
    acc.update(&3);
    assert_eq!(acc.emit(), vec![1, 2, 3]);
  }

  #[test]
  fn string_accumulator() {
    let mut acc = StringAccumulator::default();
    acc.update(&'1');
    acc.update(&'2');
    acc.update(&'3');
    assert_eq!(acc.emit(), "123");
  }
}
