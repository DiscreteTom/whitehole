use crate::lexer::action::Accumulator;

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

  #[test]
  fn integer_literal_body_string_accumulator() {
    let mut acc = IntegerLiteralBodyStringAccumulator::default();
    acc.update(&'1');
    acc.update(&'2');
    acc.update(&'3');
    assert_eq!(acc.emit(), "123");
  }
}
