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

/// Accumulate non-numeric-separator from an integer literal body
/// and emit the value of the body as a [`String`].
/// E.g. the value of `"123_456"` is `"123456"`.
#[derive(Clone, Debug, Default)]
pub struct IntegerLiteralBodyStringAccumulator(String);
impl Accumulator<char> for IntegerLiteralBodyStringAccumulator {
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
  fn integer_literal_body_string_accumulator() {
    let mut acc = IntegerLiteralBodyStringAccumulator::default();
    acc.update(&'1');
    acc.update(&'2');
    acc.update(&'3');
    assert_eq!(acc.emit(), "123");
  }
}
