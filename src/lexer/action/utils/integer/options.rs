use super::{IntegerLiteralBodySeparatorAccumulator, MockAccumulator};

pub struct IntegerLiteralBodyOptions<SepAcc, ValueAcc> {
  /// See [`Self::separator`].
  pub separator: Option<(char, SepAcc)>,
  /// See [`Self::value`].
  pub value: Option<ValueAcc>,
}

impl Default for IntegerLiteralBodyOptions<MockAccumulator, MockAccumulator> {
  fn default() -> Self {
    Self {
      separator: None,
      value: None,
    }
  }
}

impl<SepAcc, ValueAcc> IntegerLiteralBodyOptions<SepAcc, ValueAcc> {
  /// Numeric separator for the integer literal.
  /// Default is [`None`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::IntegerLiteralBodyOptions;
  /// let options = IntegerLiteralBodyOptions::default().separator('_');
  /// ```
  pub fn separator(
    self,
    separator: char,
  ) -> IntegerLiteralBodyOptions<IntegerLiteralBodySeparatorAccumulator, ValueAcc> {
    IntegerLiteralBodyOptions {
      separator: Some((separator, IntegerLiteralBodySeparatorAccumulator::default())),
      value: self.value,
    }
  }

  /// An accumulator to accumulate the integer literal body's value.
  /// Default is [`None`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions, IntegerLiteralBodyStringValueAccumulator};
  /// let options = IntegerLiteralBodyOptions::default().value(IntegerLiteralBodyStringValueAccumulator::default());
  /// ```
  pub fn value<NewAcc>(self, acc: NewAcc) -> IntegerLiteralBodyOptions<SepAcc, NewAcc> {
    IntegerLiteralBodyOptions {
      separator: self.separator,
      value: Some(acc),
    }
  }
}
