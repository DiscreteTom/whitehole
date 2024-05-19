/// Accumulate non-numeric-separator chars from an integer literal body
/// and emit the value of the body. See [`IntegerLiteralBodyStringAccumulator`].
pub trait IntegerLiteralBodyAccumulator: Clone {
  type Target: Default;

  /// Update the accumulator with a non-numeric-separator [`char`].
  fn update(&mut self, c: &char); // TODO: batch update with a String instead of one char?
  /// Consume the accumulator and emit the value of the body.
  fn emit(self) -> Self::Target;
}

/// A mock [`IntegerLiteralBodyAccumulator`] that does nothing.
/// This should only be used when
/// [`IntegerLiteralBodyOptions::acc`](super::IntegerLiteralBodyOptions::acc)
/// is [`None`].
#[derive(Clone, Debug)]
pub struct MockIntegerLiteralBodyAccumulator;
impl IntegerLiteralBodyAccumulator for MockIntegerLiteralBodyAccumulator {
  type Target = ();
  fn update(&mut self, _c: &char) {
    unreachable!("MockIntegerLiteralBodyAccumulator::update should never be called")
  }
  fn emit(self) -> Self::Target {
    unreachable!("MockIntegerLiteralBodyAccumulator::emit should never be called")
  }
}

/// Accumulate non-numeric-separator from an integer literal body
/// and emit the value of the body as a [`String`].
/// E.g. the value of `"123_456"` is `"123456"`.
#[derive(Clone, Debug, Default)]
pub struct IntegerLiteralBodyStringAccumulator(String);
impl IntegerLiteralBodyAccumulator for IntegerLiteralBodyStringAccumulator {
  type Target = String;
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
  fn mock_integer_literal_body_accumulator_update() {
    let mut acc = MockIntegerLiteralBodyAccumulator;
    acc.update(&'a');
  }

  #[should_panic]
  #[test]
  fn mock_integer_literal_body_accumulator_emit() {
    let acc = MockIntegerLiteralBodyAccumulator;
    acc.emit();
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
