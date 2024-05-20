use crate::lexer::action::Accumulator;

/// Accumulate the byte index of numeric separators from an integer literal body.
#[derive(Clone, Debug, Default)]
pub struct IntegerLiteralBodySeparatorAccumulator(Vec<usize>);
impl Accumulator<usize> for IntegerLiteralBodySeparatorAccumulator {
  type Target = Vec<usize>;
  fn update(&mut self, c: &usize) {
    self.0.push(*c);
  }
  fn emit(self) -> Self::Target {
    self.0
  }
}

/// Accumulate non-numeric-separator from an integer literal body
/// and emit the value of the body as a [`String`].
/// E.g. the value of `"123_456"` is `"123456"`.
#[derive(Clone, Debug, Default)]
pub struct IntegerLiteralBodyStringValueAccumulator(String);
impl Accumulator<char> for IntegerLiteralBodyStringValueAccumulator {
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

  #[test]
  fn integer_literal_body_separator_accumulator() {
    let mut acc = IntegerLiteralBodySeparatorAccumulator::default();
    acc.update(&1);
    acc.update(&2);
    acc.update(&3);
    assert_eq!(acc.emit(), vec![1, 2, 3]);
  }

  #[test]
  fn integer_literal_body_string_value_accumulator() {
    let mut acc = IntegerLiteralBodyStringValueAccumulator::default();
    acc.update(&'1');
    acc.update(&'2');
    acc.update(&'3');
    assert_eq!(acc.emit(), "123");
  }
}
