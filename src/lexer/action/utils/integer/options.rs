use super::MockAccumulator;
use crate::lexer::action::{Accumulator, StringAccumulator, VecAccumulator};

pub trait NumericSeparatorAccumulator: Clone {
  type Target;

  /// Return whether the character is a valid numeric separator.
  fn validate(&self, c: char) -> bool;
  /// Accumulate the index of the numeric separator.
  fn update(&mut self, c: usize);
  /// Emit the accumulated indexes.
  fn emit(self) -> Self::Target;
}

/// This struct is to indicate that no numeric separator is allowed.
#[derive(Clone, Debug, Default)]
pub struct MockNumericSeparatorAccumulator;

impl NumericSeparatorAccumulator for MockNumericSeparatorAccumulator {
  type Target = ();
  fn validate(&self, _: char) -> bool {
    false
  }
  fn update(&mut self, _: usize) {}
  fn emit(self) {}
}

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
  pub fn acc_to_vec(self) -> NumericSeparatorOptions<VecAccumulator<usize>> {
    self.acc(VecAccumulator::default())
  }
}

impl<Acc: Accumulator<usize>> NumericSeparatorAccumulator for NumericSeparatorOptions<Acc> {
  type Target = Acc::Target;
  fn validate(&self, c: char) -> bool {
    c == self.ch
  }
  fn update(&mut self, c: usize) {
    self.acc.update(c);
  }
  fn emit(self) -> Self::Target {
    self.acc.emit()
  }
}

#[derive(Clone, Debug)]
pub struct IntegerLiteralBodyOptions<SepAcc, ValueAcc> {
  /// See [`Self::separator`].
  pub separator: SepAcc,
  /// See [`Self::value`].
  pub value: ValueAcc,
}

impl Default for IntegerLiteralBodyOptions<MockNumericSeparatorAccumulator, MockAccumulator> {
  fn default() -> Self {
    Self {
      separator: MockNumericSeparatorAccumulator,
      value: MockAccumulator,
    }
  }
}

impl<SepAcc, ValueAcc> IntegerLiteralBodyOptions<SepAcc, ValueAcc> {
  /// Set the numeric separator for the integer literal.
  /// Default is [`MockNumericSeparatorAccumulator`] (no separator allowed).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions, NumericSeparatorOptions};
  /// let options = IntegerLiteralBodyOptions::default()
  ///   .separator(NumericSeparatorOptions::default());
  /// ```
  pub fn separator<Acc>(self, options: Acc) -> IntegerLiteralBodyOptions<Acc, ValueAcc> {
    IntegerLiteralBodyOptions {
      separator: options,
      value: self.value,
    }
  }

  /// Set [`Self::separator`] to [`NumericSeparatorOptions`] using the given options builder.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions};
  /// let options = IntegerLiteralBodyOptions::default()
  ///   .separator_with(|s| s.ch('-').acc_to_vec());
  /// ```
  pub fn separator_with<Acc>(
    self,
    options_builder: impl FnOnce(NumericSeparatorOptions<MockAccumulator>) -> Acc,
  ) -> IntegerLiteralBodyOptions<Acc, ValueAcc> {
    self.separator(options_builder(NumericSeparatorOptions::default()))
  }

  /// Set the numeric separator for the integer literal to the default value of
  /// [`NumericSeparatorOptions`] (use `'_'` as the separator, no accumulator).
  pub fn default_separator(
    self,
  ) -> IntegerLiteralBodyOptions<NumericSeparatorOptions<MockAccumulator>, ValueAcc> {
    self.separator(NumericSeparatorOptions::default())
  }

  /// Set an accumulator to accumulate the integer literal body's value.
  /// Default is [`MockAccumulator`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions, StringAccumulator};
  /// let options = IntegerLiteralBodyOptions::default().value(StringAccumulator::default());
  /// ```
  pub fn value<NewAcc>(self, acc: NewAcc) -> IntegerLiteralBodyOptions<SepAcc, NewAcc> {
    IntegerLiteralBodyOptions {
      separator: self.separator,
      value: acc,
    }
  }

  /// Set [`Self::value`] to [`StringAccumulator`].
  pub fn value_to_string(self) -> IntegerLiteralBodyOptions<SepAcc, StringAccumulator> {
    self.value(StringAccumulator::default())
  }
}
