use super::MockAccumulator;
use crate::lexer::action::VecAccumulator;

#[derive(Clone, Debug)]
pub struct NumericSeparatorOptions<Acc> {
  /// See [`Self::ch`].
  pub ch: char,
  /// See [`Self::acc`].
  pub acc: Acc,
}

impl Default for NumericSeparatorOptions<MockAccumulator> {
  fn default() -> Self {
    Self {
      ch: '_',
      acc: MockAccumulator,
    }
  }
}

impl<Acc> NumericSeparatorOptions<Acc> {
  /// Set the character used as the numeric separator.
  /// Default is `'_'`.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::NumericSeparatorOptions;
  /// let options = NumericSeparatorOptions::default().ch('-');
  /// ```
  pub fn ch(mut self, separator: char) -> Self {
    self.ch = separator;
    self
  }

  /// Set an accumulator to accumulate the index of numeric separators.
  /// Default is [`MockAccumulator`].
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{NumericSeparatorOptions, VecAccumulator};
  /// let options = NumericSeparatorOptions::default().acc(VecAccumulator::default());
  /// ```
  pub fn acc<NewAcc>(self, acc: NewAcc) -> NumericSeparatorOptions<NewAcc> {
    NumericSeparatorOptions { ch: self.ch, acc }
  }

  /// Set [`Self::acc`] to [`VecAccumulator`].
  pub fn acc_to_vec(self) -> NumericSeparatorOptions<VecAccumulator> {
    self.acc(VecAccumulator::default())
  }
}

#[derive(Clone, Debug)]
pub struct IntegerLiteralBodyOptions<SepAcc, ValueAcc> {
  /// See [`Self::separator`].
  pub separator: Option<NumericSeparatorOptions<SepAcc>>,
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
  /// Set the numeric separator for the integer literal.
  /// Default is [`None`] (no separator allowed).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions, NumericSeparatorOptions};
  /// let options = IntegerLiteralBodyOptions::default().separator(NumericSeparatorOptions::default());
  /// ```
  pub fn separator<Acc>(
    self,
    options: NumericSeparatorOptions<Acc>,
  ) -> IntegerLiteralBodyOptions<Acc, ValueAcc> {
    IntegerLiteralBodyOptions {
      separator: Some(options),
      value: self.value,
    }
  }

  /// Set the numeric separator for the integer literal.
  /// Default is [`None`] (no separator allowed).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions};
  /// let options = IntegerLiteralBodyOptions::default()
  ///   .separator_with(|s| s.ch('-').acc_to_vec());
  /// ```
  pub fn separator_with<Acc>(
    self,
    options_builder: impl FnOnce(
      NumericSeparatorOptions<MockAccumulator>,
    ) -> NumericSeparatorOptions<Acc>,
  ) -> IntegerLiteralBodyOptions<Acc, ValueAcc> {
    self.separator(options_builder(NumericSeparatorOptions::default()))
  }

  /// Set the numeric separator for the integer literal to the default.
  pub fn default_separator(self) -> IntegerLiteralBodyOptions<MockAccumulator, ValueAcc> {
    self.separator(NumericSeparatorOptions::default())
  }

  /// Set an accumulator to accumulate the integer literal body's value.
  /// Default is [`None`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions, StringAccumulator};
  /// let options = IntegerLiteralBodyOptions::default().value(StringAccumulator::default());
  /// ```
  pub fn value<NewAcc>(self, acc: NewAcc) -> IntegerLiteralBodyOptions<SepAcc, NewAcc> {
    IntegerLiteralBodyOptions {
      separator: self.separator,
      value: Some(acc),
    }
  }
}
