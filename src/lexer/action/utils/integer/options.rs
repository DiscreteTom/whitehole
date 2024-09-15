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

impl Default for NumericSeparatorOptions<()> {
  fn default() -> Self {
    Self::new()
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

impl Default for IntegerLiteralBodyOptions<(), ()> {
  fn default() -> Self {
    Self::new()
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
  pub fn separator<Acc>(
    self,
    options: NumericSeparatorOptions<Acc>,
  ) -> IntegerLiteralBodyOptions<NumericSeparatorOptions<Acc>, ValueAcc> {
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
    options_builder: impl FnOnce(NumericSeparatorOptions<()>) -> NumericSeparatorOptions<Acc>,
  ) -> IntegerLiteralBodyOptions<NumericSeparatorOptions<Acc>, ValueAcc> {
    self.separator(options_builder(NumericSeparatorOptions::new()))
  }

  /// Enable numeric separator with `'_'` as the separator and no index accumulator.
  #[inline]
  pub fn default_separator(
    self,
  ) -> IntegerLiteralBodyOptions<NumericSeparatorOptions<()>, ValueAcc> {
    self.separator(NumericSeparatorOptions::new())
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

#[cfg(test)]
mod tests {
  use super::*;

  /// Assert the unit type.
  macro_rules! assert_unit {
    ($e: expr) => {
      let _: () = $e;
    };
  }

  #[test]
  fn test_mock_numeric_separator_accumulator() {
    fn test(acc: impl NumericSeparatorAccumulator<Acc = ()>) {
      assert!(!acc.validate('_'));
      assert!(!acc.validate('1'));
      assert_unit!(acc.emit());
    }
    test(());
  }

  #[test]
  fn test_numeric_separator_options() {
    // new
    let mut options = NumericSeparatorOptions::new();
    assert_eq!(options.char, '_');
    assert_unit!(options.indexes_to);

    // methods
    options = options.char('-');
    assert_eq!(options.char, '-');
    let options = options.indexes_to(vec![()]);
    assert_eq!(options.indexes_to, vec![()]);
    let mut options = options.indexes_to_vec();
    assert_eq!(options.indexes_to, vec![]);

    // impl
    assert!(options.validate('-'));
    assert!(!options.validate('_'));
    options.update(0);
    assert_eq!(options.emit(), vec![0]);
  }

  #[test]
  fn test_integer_literal_body_options() {
    // new
    let options = IntegerLiteralBodyOptions::new();
    assert_unit!(options.separator);
    assert_unit!(options.value_to);

    // methods
    let options = options.separator(NumericSeparatorOptions::new());
    assert_eq!(options.separator.char, '_');
    assert_unit!(options.separator.indexes_to);
    let options = options.separator_with(|s| s.char('-').indexes_to_vec());
    assert_eq!(options.separator.char, '-');
    assert_eq!(options.separator.indexes_to, vec![]);
    let options = options.default_separator();
    assert_eq!(options.separator.char, '_');
    assert_unit!(options.separator.indexes_to);
    let options = options.value_to(String::new());
    assert_eq!(options.value_to, String::new());
    let options = options.value_to_string();
    assert_eq!(options.value_to, String::new());
  }
}
