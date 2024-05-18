use super::MockIntegerLiteralBodyAccumulator;

pub struct IntegerLiteralBodyOptions<Acc> {
  /// See [`Self::sep`].
  pub sep: Option<char>,
  /// See [`Self::acc`].
  pub acc: Option<Acc>,
}

impl Default for IntegerLiteralBodyOptions<MockIntegerLiteralBodyAccumulator> {
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
  /// # use whitehole::lexer::action::IntegerLiteralOptions;
  /// let options = IntegerLiteralOptions::default().sep('_');
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
  /// # use whitehole::lexer::action::IntegerLiteralOptions;
  /// let options = IntegerLiteralOptions::default().acc(|c| {
  /// # // TODO: update this example, calculate a value here
  ///   println!("{}", c);
  /// });
  /// ```
  pub fn acc<NewAcc>(self, acc: NewAcc) -> IntegerLiteralBodyOptions<NewAcc> {
    IntegerLiteralBodyOptions {
      sep: self.sep,
      acc: Some(acc),
    }
  }
}
