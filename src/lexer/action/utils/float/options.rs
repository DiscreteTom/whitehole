use std::collections::HashSet;

use crate::{lexer::action::NumericSeparatorOptions, utils::OneOrMore};

#[derive(Clone, Debug)]
pub struct FloatFractionOptions<Acc> {
  /// See [`Self::point`].
  pub point: char,
  /// See [`Self::acc`].
  pub acc: Acc,
}

impl Default for FloatFractionOptions<()> {
  fn default() -> Self {
    Self {
      point: '.',
      acc: (),
    }
  }
}

impl<Acc> FloatFractionOptions<Acc> {
  /// Set the character used as the decimal point.
  /// Default is `'.'`.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::FloatFractionOptions;
  /// let options = FloatFractionOptions::default().point(',');
  /// ```
  pub fn point(mut self, point: char) -> Self {
    self.point = point;
    self
  }

  /// Set an accumulator to accumulate the fractional part.
  /// Default is [`()`].
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatFractionOptions};
  /// let options = FloatFractionOptions::default().acc(String::new());
  /// ```
  pub fn acc<NewAcc>(self, acc: NewAcc) -> FloatFractionOptions<NewAcc> {
    FloatFractionOptions {
      point: self.point,
      acc,
    }
  }

  // TODO: abstract a trait for `acc` and impl this in trait.
  /// Set [`Self::acc`] to [`String`].
  pub fn acc_to_string(self) -> FloatFractionOptions<String> {
    self.acc(String::new())
  }
}

#[derive(Clone, Debug)]
pub struct FloatExponentOptions<Acc> {
  /// See [`Self::indicators`].
  pub indicators: Vec<String>,
  indicator_heads: HashSet<char>,
  /// See [`Self::acc`].
  pub acc: Acc,
}

impl Default for FloatExponentOptions<()> {
  fn default() -> Self {
    Self {
      indicators: vec!["e-", "e+", "e", "E-", "E+", "E"]
        .iter()
        .map(|s| s.to_string())
        .collect(),
      acc: (),
      indicator_heads: vec!['e', 'E'].into_iter().collect(),
    }
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
  /// let options = FloatExponentOptions::default().indicators(["e", "E"]);
  /// ```
  pub fn indicators(mut self, indicators: impl Into<OneOrMore<String>>) -> Self {
    self.indicators = indicators.into().0;
    self.indicator_heads = self
      .indicators
      .iter()
      .map(|s| s.chars().next().unwrap())
      .collect();
    self
  }

  pub(super) fn indicator_heads(&self) -> &HashSet<char> {
    &self.indicator_heads
  }

  /// Set an accumulator to accumulate the exponent part.
  /// Default is [`()`].
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatExponentOptions};
  /// let options = FloatExponentOptions::default().acc(String::default());
  /// ```
  pub fn acc<NewAcc>(self, acc: NewAcc) -> FloatExponentOptions<NewAcc> {
    FloatExponentOptions {
      indicators: self.indicators,
      indicator_heads: self.indicator_heads,
      acc,
    }
  }

  /// Set [`Self::acc`] to [`String`].
  pub fn acc_to_string(self) -> FloatExponentOptions<String> {
    self.acc(String::default())
  }
}

#[derive(Clone, Debug)]
pub struct FloatLiteralOptions<Sep, IntAcc, FracAcc, ExpAcc> {
  /// See [`Self::separator`].
  pub separator: Sep,
  /// See [`Self::integer`].
  pub integer: IntAcc,
  /// See [`Self::fraction`].
  pub fraction: Option<FloatFractionOptions<FracAcc>>,
  /// See [`Self::exponent`].
  pub exponent: Option<FloatExponentOptions<ExpAcc>>,
}

impl Default for FloatLiteralOptions<(), (), (), ()> {
  fn default() -> Self {
    Self {
      separator: (),
      integer: (),
      // use `None` to disable the optional parts
      fraction: None,
      exponent: None,
    }
  }
}

impl<Sep, IntAcc, FracAcc, ExpAcc> FloatLiteralOptions<Sep, IntAcc, FracAcc, ExpAcc> {
  /// Set the accumulator for the integer part.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatLiteralOptions};
  /// let options = FloatLiteralOptions::default().integer(String::default());
  /// ```
  pub fn integer<NewIntAcc>(
    self,
    integer: NewIntAcc,
  ) -> FloatLiteralOptions<Sep, NewIntAcc, FracAcc, ExpAcc> {
    FloatLiteralOptions {
      integer,
      fraction: self.fraction,
      exponent: self.exponent,
      separator: self.separator,
    }
  }

  /// Set [`Self::integer`] to [`String`].
  pub fn integer_to_string(self) -> FloatLiteralOptions<Sep, String, FracAcc, ExpAcc> {
    self.integer(String::default())
  }

  /// Set the accumulator for the fractional part.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatLiteralOptions, FloatFractionOptions};
  /// let options = FloatLiteralOptions::default().fraction(FloatFractionOptions::default());
  /// ```
  pub fn fraction<NewFracAcc>(
    self,
    fraction: FloatFractionOptions<NewFracAcc>,
  ) -> FloatLiteralOptions<Sep, IntAcc, NewFracAcc, ExpAcc> {
    FloatLiteralOptions {
      integer: self.integer,
      fraction: Some(fraction),
      exponent: self.exponent,
      separator: self.separator,
    }
  }

  /// Set [`Self::fraction`] to [`FloatFractionOptions`] using the given options builder.
  pub fn fraction_with<NewFracAcc>(
    self,
    options_builder: impl FnOnce(FloatFractionOptions<()>) -> FloatFractionOptions<NewFracAcc>,
  ) -> FloatLiteralOptions<Sep, IntAcc, NewFracAcc, ExpAcc> {
    self.fraction(options_builder(FloatFractionOptions::default()))
  }

  /// Set [`Self::fraction`] to the default value of [`FloatFractionOptions`].
  pub fn default_fraction(self) -> FloatLiteralOptions<Sep, IntAcc, (), ExpAcc> {
    self.fraction(FloatFractionOptions::default())
  }

  /// Set the accumulator for the exponent part.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatLiteralOptions, FloatExponentOptions};
  /// let options = FloatLiteralOptions::default().exponent(FloatExponentOptions::default());
  /// ```
  pub fn exponent<NewExpAcc>(
    self,
    exponent: FloatExponentOptions<NewExpAcc>,
  ) -> FloatLiteralOptions<Sep, IntAcc, FracAcc, NewExpAcc> {
    FloatLiteralOptions {
      integer: self.integer,
      fraction: self.fraction,
      exponent: Some(exponent),
      separator: self.separator,
    }
  }

  /// Set [`Self::exponent`] to [`FloatExponentOptions`] using the given options builder.
  /// # Examples
  /// ```rust
  /// # use whitehole::lexer::action::{FloatLiteralOptions, FloatExponentOptions};
  /// let options = FloatLiteralOptions::default().exponent_with(|e| e.acc_to_string());
  /// ```
  pub fn exponent_with<NewExpAcc>(
    self,
    options_builder: impl FnOnce(FloatExponentOptions<()>) -> FloatExponentOptions<NewExpAcc>,
  ) -> FloatLiteralOptions<Sep, IntAcc, FracAcc, NewExpAcc> {
    self.exponent(options_builder(FloatExponentOptions::default()))
  }

  /// Set [`Self::exponent`] to the default value of [`FloatExponentOptions`].
  pub fn default_exponent(self) -> FloatLiteralOptions<Sep, IntAcc, FracAcc, ()> {
    self.exponent(FloatExponentOptions::default())
  }

  /// Set the numeric separator for the float literal.
  /// Default is [`()`] (no separator allowed).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{FloatLiteralOptions, NumericSeparatorOptions};
  /// let options = FloatLiteralOptions::default()
  ///   .separator(NumericSeparatorOptions::new());
  /// ```
  pub fn separator<NewSep>(
    self,
    options: NewSep,
  ) -> FloatLiteralOptions<NewSep, IntAcc, FracAcc, ExpAcc> {
    FloatLiteralOptions {
      integer: self.integer,
      fraction: self.fraction,
      exponent: self.exponent,
      separator: options,
    }
  }

  /// Set [`Self::separator`] to [`NumericSeparatorOptions`] using the given options builder.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{FloatLiteralOptions};
  /// let options = FloatLiteralOptions::default()
  ///   .separator_with(|s| s.char('-').indexes_to_vec());
  /// ```
  pub fn separator_with<NewSep>(
    self,
    options_builder: impl FnOnce(NumericSeparatorOptions<()>) -> NewSep,
  ) -> FloatLiteralOptions<NewSep, IntAcc, FracAcc, ExpAcc> {
    self.separator(options_builder(NumericSeparatorOptions::new()))
  }

  /// Set the numeric separator for the float literal to the default value of
  /// [`NumericSeparatorOptions`] (use `'_'` as the separator, no accumulator).
  pub fn default_separator(
    self,
  ) -> FloatLiteralOptions<NumericSeparatorOptions<()>, IntAcc, FracAcc, ExpAcc> {
    self.separator(NumericSeparatorOptions::new())
  }
}
