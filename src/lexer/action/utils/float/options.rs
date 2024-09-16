use crate::{lexer::action::NumericSeparatorOptions, utils::OneOrMore};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct FloatFractionalOptions<Acc> {
  /// See [`Self::point`].
  pub point: char,
  /// See [`Self::value_to`].
  pub value_to: Acc,
}

impl FloatFractionalOptions<()> {
  /// Create a new instance with `'.'` as the decimal point and no accumulator.
  #[inline]
  pub const fn new() -> Self {
    Self {
      point: '.',
      value_to: (),
    }
  }
}

impl Default for FloatFractionalOptions<()> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<Acc> FloatFractionalOptions<Acc> {
  /// Set the character used as the decimal point.
  /// Default is `'.'`.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::FloatFractionalOptions;
  /// let options = FloatFractionalOptions::new().point(',');
  /// ```
  #[inline]
  pub const fn point(mut self, point: char) -> Self {
    self.point = point;
    self
  }

  /// Set an accumulator to accumulate the fractional part.
  /// Default is `()` which means no accumulator.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatFractionalOptions};
  /// let options = FloatFractionalOptions::new().value_to(String::new());
  /// ```
  #[inline]
  pub fn value_to<NewAcc>(self, acc: NewAcc) -> FloatFractionalOptions<NewAcc> {
    FloatFractionalOptions {
      point: self.point,
      value_to: acc,
    }
  }

  // TODO: abstract a trait for `value_to` and impl `value_to_string` in trait?
  /// Accumulate the fractional part's value into a [`String`].
  #[inline]
  pub fn value_to_string(self) -> FloatFractionalOptions<String> {
    self.value_to(String::new())
  }
}

#[derive(Clone, Debug)]
pub struct FloatExponentOptions<Acc> {
  /// See [`Self::indicators`].
  pub indicators: Vec<String>,
  indicator_heads: HashSet<char>,
  /// See [`Self::value_to`].
  pub value_to: Acc,
}

impl FloatExponentOptions<()> {
  /// Create a new instance with the candidate strings `["e-", "e+", "e", "E-", "E+", "E"]` and no accumulator.
  pub fn new() -> Self {
    Self {
      indicators: ["e-", "e+", "e", "E-", "E+", "E"]
        .iter()
        .map(|s| s.to_string())
        .collect(),
      value_to: (),
      indicator_heads: vec!['e', 'E'].into_iter().collect(),
    }
  }
}

impl Default for FloatExponentOptions<()> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<Acc> FloatExponentOptions<Acc> {
  /// Set the candidate strings used as the exponent indicator.
  /// Default is `["e-", "e+", "e", "E-", "E+", "E"]`.
  /// # Caveats
  /// Candidates are checked in order, so make sure the longer ones are placed first.
  /// E.g. `"e-"` should be placed before `"e"`.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::FloatExponentOptions;
  /// let options = FloatExponentOptions::new().indicators(["e", "E"]);
  /// ```
  pub fn indicators(mut self, indicators: impl Into<OneOrMore<String>>) -> Self {
    self.indicators = indicators.into().0;
    self.indicator_heads = self
      .indicators
      .iter()
      .map(|s| s.chars().next().expect("empty exponent indicator"))
      .collect();
    self
  }

  #[inline]
  pub(super) fn indicator_heads(&self) -> &HashSet<char> {
    &self.indicator_heads
  }

  /// Set an accumulator to accumulate the exponent part.
  /// Default is `()`.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatExponentOptions};
  /// let options = FloatExponentOptions::new().value_to(String::default());
  /// ```
  #[inline]
  pub fn value_to<NewAcc>(self, acc: NewAcc) -> FloatExponentOptions<NewAcc> {
    FloatExponentOptions {
      indicators: self.indicators,
      indicator_heads: self.indicator_heads,
      value_to: acc,
    }
  }

  /// Accumulate the exponent part's value into a [`String`].
  #[inline]
  pub fn value_to_string(self) -> FloatExponentOptions<String> {
    self.value_to(String::default())
  }
}

#[derive(Clone, Debug)]
pub struct FloatLiteralOptions<Sep, IntAcc, FracAcc, ExpAcc> {
  /// See [`Self::separator`].
  pub separator: Sep,
  /// See [`Self::integral_to`].
  pub integral_to: IntAcc,
  /// See [`Self::fractional`].
  pub fractional: Option<FloatFractionalOptions<FracAcc>>,
  /// See [`Self::exponent`].
  pub exponent: Option<FloatExponentOptions<ExpAcc>>,
}

impl FloatLiteralOptions<(), (), (), ()> {
  /// Create a new instance with no accumulator for any part,
  /// and the fractional and exponent parts disabled.
  #[inline]
  pub const fn new() -> Self {
    Self {
      separator: (),
      integral_to: (),
      // use `None` to disable the optional parts
      fractional: None,
      exponent: None,
    }
  }
}

impl Default for FloatLiteralOptions<(), (), (), ()> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<Sep, IntAcc, FracAcc, ExpAcc> FloatLiteralOptions<Sep, IntAcc, FracAcc, ExpAcc> {
  /// Set the accumulator for the integral part.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatLiteralOptions};
  /// let options = FloatLiteralOptions::new().integral_to(String::default());
  /// ```
  #[inline]
  pub fn integral_to<NewIntAcc>(
    self,
    acc: NewIntAcc,
  ) -> FloatLiteralOptions<Sep, NewIntAcc, FracAcc, ExpAcc> {
    FloatLiteralOptions {
      integral_to: acc,
      fractional: self.fractional,
      exponent: self.exponent,
      separator: self.separator,
    }
  }

  /// Accumulate the integral part's value into a [`String`].
  #[inline]
  pub fn integral_to_string(self) -> FloatLiteralOptions<Sep, String, FracAcc, ExpAcc> {
    self.integral_to(String::default())
  }

  /// Enable the fractional part with the given decimal point and the accumulator for the fractional part.
  /// By default the fractional part is disabled.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatLiteralOptions, FloatFractionalOptions};
  /// let options = FloatLiteralOptions::new().fractional(FloatFractionalOptions::new());
  /// ```
  #[inline]
  pub fn fractional<NewFracAcc>(
    self,
    fraction: impl Into<FloatFractionalOptions<NewFracAcc>>,
  ) -> FloatLiteralOptions<Sep, IntAcc, NewFracAcc, ExpAcc> {
    FloatLiteralOptions {
      integral_to: self.integral_to,
      fractional: Some(fraction.into()),
      exponent: self.exponent,
      separator: self.separator,
    }
  }

  /// Enable the fractional part using the given options builder.
  /// By default the fractional part is disabled.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatLiteralOptions};
  /// let options = FloatLiteralOptions::new().fractional_with(|o| o.point(','));
  #[inline]
  pub fn fractional_with<NewFracAcc>(
    self,
    options_builder: impl FnOnce(FloatFractionalOptions<()>) -> FloatFractionalOptions<NewFracAcc>,
  ) -> FloatLiteralOptions<Sep, IntAcc, NewFracAcc, ExpAcc> {
    self.fractional(options_builder(FloatFractionalOptions::new()))
  }

  /// Enable the fractional part with `'.'` as the decimal point and no accumulator.
  #[inline]
  pub fn default_fractional(self) -> FloatLiteralOptions<Sep, IntAcc, (), ExpAcc> {
    self.fractional(FloatFractionalOptions::new())
  }

  /// Enable the exponent part with the given exponent indicator strings and the accumulator for the exponent part.
  /// By default the exponent part is disabled.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatLiteralOptions, FloatExponentOptions};
  /// let options = FloatLiteralOptions::new().exponent(FloatExponentOptions::new());
  /// ```
  #[inline]
  pub fn exponent<NewExpAcc>(
    self,
    exponent: FloatExponentOptions<NewExpAcc>,
  ) -> FloatLiteralOptions<Sep, IntAcc, FracAcc, NewExpAcc> {
    FloatLiteralOptions {
      integral_to: self.integral_to,
      fractional: self.fractional,
      exponent: Some(exponent),
      separator: self.separator,
    }
  }

  /// Enable the exponent part using the given options builder.
  /// By default the exponent part is disabled.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatLiteralOptions, FloatExponentOptions};
  /// let options = FloatLiteralOptions::new().exponent_with(|e| e.value_to_string());
  /// ```
  #[inline]
  pub fn exponent_with<NewExpAcc>(
    self,
    options_builder: impl FnOnce(FloatExponentOptions<()>) -> FloatExponentOptions<NewExpAcc>,
  ) -> FloatLiteralOptions<Sep, IntAcc, FracAcc, NewExpAcc> {
    self.exponent(options_builder(FloatExponentOptions::new()))
  }

  /// Enable the exponent part with `["e-", "e+", "e", "E-", "E+", "E"]` as the indicator strings,
  /// no accumulator.
  #[inline]
  pub fn default_exponent(self) -> FloatLiteralOptions<Sep, IntAcc, FracAcc, ()> {
    self.exponent(FloatExponentOptions::new())
  }

  /// Set the numeric separator for the float literal.
  /// By default the numeric separator is disabled.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{FloatLiteralOptions, NumericSeparatorOptions};
  /// let options = FloatLiteralOptions::new()
  ///   .separator(NumericSeparatorOptions::new());
  /// ```
  #[inline]
  pub fn separator<NewSep>(
    self,
    options: NewSep,
  ) -> FloatLiteralOptions<NewSep, IntAcc, FracAcc, ExpAcc> {
    FloatLiteralOptions {
      integral_to: self.integral_to,
      fractional: self.fractional,
      exponent: self.exponent,
      separator: options,
    }
  }

  /// Enable separator with [`NumericSeparatorOptions`] using the given options builder.
  /// By default the numeric separator is disabled.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{FloatLiteralOptions};
  /// let options = FloatLiteralOptions::new()
  ///   .separator_with(|s| s.char('-').indexes_to_vec());
  /// ```
  #[inline]
  pub fn separator_with<NewSep>(
    self,
    options_builder: impl FnOnce(NumericSeparatorOptions<()>) -> NewSep,
  ) -> FloatLiteralOptions<NewSep, IntAcc, FracAcc, ExpAcc> {
    self.separator(options_builder(NumericSeparatorOptions::new()))
  }

  /// Enable separator with `'_'` as the separator and no index accumulator.
  #[inline]
  pub fn default_separator(
    self,
  ) -> FloatLiteralOptions<NumericSeparatorOptions<()>, IntAcc, FracAcc, ExpAcc> {
    self.separator(NumericSeparatorOptions::new())
  }
}
