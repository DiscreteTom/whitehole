use super::MockAccumulator;

pub struct IntegerLiteralBodyOptions<Acc> {
  /// See [`Self::sep`].
  pub sep: Option<char>,
  /// See [`Self::acc`].
  pub acc: Option<Acc>,
}

impl Default for IntegerLiteralBodyOptions<MockAccumulator> {
  fn default() -> Self {
    Self {
      sep: None,
      acc: None,
    }
  }
}

impl<Acc> IntegerLiteralBodyOptions<Acc> {
  /// Numeric separator for the integer literal.
  /// Default is [`None`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::IntegerLiteralBodyOptions;
  /// let options = IntegerLiteralBodyOptions::default().sep('_');
  /// ```
  pub fn sep(mut self, separator: char) -> Self {
    self.sep = Some(separator);
    self
  }

  /// Accumulator when lexing the integer literal.
  /// You can use this to calculate the integer literal's value
  /// while lexing the integer literal.
  /// Default is [`None`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions, IntegerLiteralBodyStringAccumulator};
  /// let options = IntegerLiteralBodyOptions::default().acc(IntegerLiteralBodyStringAccumulator::default());
  /// ```
  pub fn acc<NewAcc>(self, acc: NewAcc) -> IntegerLiteralBodyOptions<NewAcc> {
    IntegerLiteralBodyOptions {
      sep: self.sep,
      acc: Some(acc),
    }
  }
}
