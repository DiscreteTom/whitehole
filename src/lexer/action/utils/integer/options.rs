use crate::utils::Accumulator;

/// [`Accumulator`] that can also verify whether a character is a valid numeric separator.
///
/// These types already implement the [`NumericSeparatorAccumulator`] trait:
/// - `()` - mock accumulator that does nothing. This also means the numeric separator is disabled.
/// - [`NumericSeparatorOptions`] - check for separators and accumulate the index of numeric separators.
pub trait NumericSeparatorAccumulator: Accumulator<usize> {
  /// The underlying accumulator type.
  type Acc;
  /// Return whether the character is a valid numeric separator.
  /// If return `true`, the character will be used to update the accumulator.
  fn validate(&self, c: char) -> bool;
  /// Consume self, emit the underlying accumulator.
  fn emit(self) -> Self::Acc;
}

impl NumericSeparatorAccumulator for () {
  type Acc = ();
  #[inline]
  fn validate(&self, _: char) -> bool {
    // `()` means the numeric separator is disabled,
    // nothing will be treated as a valid separator
    // so always return false
    false
  }
  #[inline]
  fn emit(self) -> Self::Acc {}
}

/// This struct indicates the numeric separator is enabled.
/// It will check whether the character is the separator and accumulate the index of separators.
#[derive(Clone, Debug)]
pub struct NumericSeparatorOptions<Acc> {
  /// See [`Self::char`].
  pub char: char,
  /// See [`Self::indexes_to`].
  pub indexes_to: Acc,
}

impl NumericSeparatorOptions<()> {
  /// Create a new instance with the separator character `'_'` and no index accumulator.
  #[inline]
  pub const fn new() -> Self {
    Self {
      char: '_',
      indexes_to: (),
    }
  }
}

impl<Acc> NumericSeparatorOptions<Acc> {
  /// Set the character used as the numeric separator.
  /// Default is `'_'`.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::NumericSeparatorOptions;
  /// let options = NumericSeparatorOptions::new().char('-');
  /// ```
  #[inline]
  pub const fn char(mut self, separator: char) -> Self {
    self.char = separator;
    self
  }

  /// Set an accumulator to accumulate the index of numeric separators.
  /// Default is `()` which means no accumulator.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{NumericSeparatorOptions};
  /// # let options: NumericSeparatorOptions<Vec<usize>> =
  /// NumericSeparatorOptions::new().indexes_to(vec![]);
  /// ```
  #[inline]
  pub fn indexes_to<NewAcc>(self, acc: NewAcc) -> NumericSeparatorOptions<NewAcc> {
    NumericSeparatorOptions {
      char: self.char,
      indexes_to: acc,
    }
  }

  /// Accumulate the index of numeric separators into a vector.
  #[inline]
  pub fn indexes_to_vec(self) -> NumericSeparatorOptions<Vec<usize>> {
    self.indexes_to(vec![])
  }
}

impl<Acc: Accumulator<usize>> NumericSeparatorAccumulator for NumericSeparatorOptions<Acc> {
  type Acc = Acc;
  #[inline]
  fn validate(&self, c: char) -> bool {
    c == self.char
  }
  #[inline]
  fn emit(self) -> Self::Acc {
    self.indexes_to
  }
}
impl<Acc: Accumulator<usize>> Accumulator<usize> for NumericSeparatorOptions<Acc> {
  #[inline]
  fn update(&mut self, c: usize) {
    self.indexes_to.update(c);
  }
}

#[derive(Debug, Clone)]
pub struct IntegerLiteralBodyOptions<SepAcc, ValueAcc> {
  /// See [`Self::separator`].
  pub separator: SepAcc,
  /// See [`Self::value_to`].
  pub value_to: ValueAcc,
}

impl IntegerLiteralBodyOptions<(), ()> {
  /// Create a new instance with numeric separator disabled and no value accumulator.
  #[inline]
  pub const fn new() -> Self {
    Self {
      separator: (),
      value_to: (),
    }
  }
}

impl<SepAcc, ValueAcc> IntegerLiteralBodyOptions<SepAcc, ValueAcc> {
  /// Set the numeric separator for the integer literal.
  /// Default is `()` which means numeric separator is disabled.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions, NumericSeparatorOptions};
  /// let options = IntegerLiteralBodyOptions::new()
  ///   .separator(NumericSeparatorOptions::new());
  /// ```
  #[inline]
  pub fn separator<Acc>(self, options: Acc) -> IntegerLiteralBodyOptions<Acc, ValueAcc> {
    IntegerLiteralBodyOptions {
      separator: options,
      value_to: self.value_to,
    }
  }

  /// Enable numeric separator with [`NumericSeparatorOptions`] using the given options builder.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions};
  /// let options = IntegerLiteralBodyOptions::new()
  ///   .separator_with(|s| s.char('-').indexes_to_vec());
  /// ```
  #[inline]
  pub fn separator_with<Acc>(
    self,
    options_builder: impl FnOnce(NumericSeparatorOptions<()>) -> Acc,
  ) -> IntegerLiteralBodyOptions<Acc, ValueAcc> {
    self.separator(options_builder(NumericSeparatorOptions::new()))
  }

  /// Set an accumulator to accumulate the integer literal body's value.
  /// Default is `()` which means no accumulator.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{IntegerLiteralBodyOptions};
  /// let options = IntegerLiteralBodyOptions::new().value_to(String::new());
  /// ```
  #[inline]
  pub fn value_to<NewAcc>(self, acc: NewAcc) -> IntegerLiteralBodyOptions<SepAcc, NewAcc> {
    IntegerLiteralBodyOptions {
      separator: self.separator,
      value_to: acc,
    }
  }

  /// Accumulate the integer literal body's value into a string.
  #[inline]
  pub fn value_to_string(self) -> IntegerLiteralBodyOptions<SepAcc, String> {
    self.value_to(String::new())
  }
}
