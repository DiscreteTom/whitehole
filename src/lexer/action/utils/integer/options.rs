use super::MockAccumulator;

pub struct IntegerLiteralBodyOptions<ValueAcc> {
  /// See [`Self::separator`].
  pub separator: Option<char>,
  /// See [`Self::value`].
  pub value: Option<ValueAcc>,
}

impl Default for IntegerLiteralBodyOptions<MockAccumulator> {
  fn default() -> Self {
    Self {
      separator: None,
      value: None,
    }
  }
}

impl<ValueAcc> IntegerLiteralBodyOptions<ValueAcc> {
  /// Numeric separator for the integer literal.
  /// Default is [`None`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::IntegerLiteralBodyOptions;
  /// let options = IntegerLiteralBodyOptions::default().separator('_');
  /// ```
  pub fn separator(mut self, separator: char) -> Self {
    self.separator = Some(separator);
    self
  }

  /// An accumulator to accumulate the integer literal body's value.
  /// Default is [`None`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions, IntegerLiteralBodyStringAccumulator};
  /// let options = IntegerLiteralBodyOptions::default().value(IntegerLiteralBodyStringAccumulator::default());
  /// ```
  pub fn value<NewAcc>(self, acc: NewAcc) -> IntegerLiteralBodyOptions<NewAcc> {
    IntegerLiteralBodyOptions {
      separator: self.separator,
      value: Some(acc),
    }
  }
}
