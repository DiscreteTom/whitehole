use crate::utils::Accumulator;

pub trait NumericSeparatorAccumulator: Accumulator<usize> {
  /// The underlying accumulator type.
  type Acc;
  /// Return whether the character is a valid numeric separator.
  fn validate(&self, c: char) -> bool;
  /// Emit the underlying accumulator.
  fn emit(self) -> Self::Acc;
}

impl NumericSeparatorAccumulator for () {
  type Acc = ();
  fn validate(&self, _: char) -> bool {
    // TODO: explain why false
    false
  }
  fn emit(self) -> Self::Acc {}
}

#[derive(Clone, Debug)]
pub struct NumericSeparatorOptions<Acc> {
  /// See [`Self::ch`].
  pub ch: char,
  /// See [`Self::acc`].
  pub acc: Acc,
}

impl Default for NumericSeparatorOptions<()> {
  fn default() -> Self {
    Self { ch: '_', acc: () }
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
  /// Default is [`()`].
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{NumericSeparatorOptions};
  /// # let options: NumericSeparatorOptions<Vec<usize>> =
  /// NumericSeparatorOptions::default().acc(vec![]);
  /// ```
  pub fn acc<NewAcc>(self, acc: NewAcc) -> NumericSeparatorOptions<NewAcc> {
    NumericSeparatorOptions { ch: self.ch, acc }
  }

  /// Set [`Self::acc`] to [`Vec`].
  pub fn acc_to_vec(self) -> NumericSeparatorOptions<Vec<usize>> {
    self.acc(vec![])
  }
}

impl<Acc: Accumulator<usize>> NumericSeparatorAccumulator for NumericSeparatorOptions<Acc> {
  type Acc = Acc;
  fn validate(&self, c: char) -> bool {
    c == self.ch
  }
  fn emit(self) -> Self::Acc {
    self.acc
  }
}
impl<Acc: Accumulator<usize>> Accumulator<usize> for NumericSeparatorOptions<Acc> {
  fn update(&mut self, c: usize) {
    self.acc.update(c);
  }
}

#[derive(Debug, Clone)]
pub struct IntegerLiteralBodyOptions<SepAcc, ValueAcc> {
  /// See [`Self::separator`].
  pub separator: SepAcc,
  /// See [`Self::value`].
  pub value: ValueAcc,
}

impl Default for IntegerLiteralBodyOptions<(), ()> {
  fn default() -> Self {
    Self {
      separator: (),
      value: (),
    }
  }
}

impl<SepAcc, ValueAcc> IntegerLiteralBodyOptions<SepAcc, ValueAcc> {
  /// Set the numeric separator for the integer literal.
  /// Default is [`()`] (no separator allowed).
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
    options_builder: impl FnOnce(NumericSeparatorOptions<()>) -> Acc,
  ) -> IntegerLiteralBodyOptions<Acc, ValueAcc> {
    self.separator(options_builder(NumericSeparatorOptions::default()))
  }

  /// Set the numeric separator for the integer literal to the default value of
  /// [`NumericSeparatorOptions`] (use `'_'` as the separator, no accumulator).
  pub fn default_separator(
    self,
  ) -> IntegerLiteralBodyOptions<NumericSeparatorOptions<()>, ValueAcc> {
    self.separator(NumericSeparatorOptions::default())
  }

  /// Set an accumulator to accumulate the integer literal body's value.
  /// Default is [`()`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions};
  /// let options = IntegerLiteralBodyOptions::default().value(String::new());
  /// ```
  pub fn value<NewAcc>(self, acc: NewAcc) -> IntegerLiteralBodyOptions<SepAcc, NewAcc> {
    IntegerLiteralBodyOptions {
      separator: self.separator,
      value: acc,
    }
  }

  /// Set [`Self::value`] to [`String`].
  pub fn value_to_string(self) -> IntegerLiteralBodyOptions<SepAcc, String> {
    self.value(String::new())
  }
}
