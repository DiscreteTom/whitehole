mod data;
mod options;

pub use data::*;
pub use options::*;

use super::{
  decimal_integer_literal_body_with_options, Accumulator, IntegerLiteralBodyOptions,
  MockAccumulator, MockNumericSeparatorAccumulator, NumericSeparatorAccumulator,
};
use crate::lexer::{
  action::{simple_with_data, Action},
  token::MockTokenKind,
};
use std::collections::HashSet;

/// Try to match a float point literal in the rest of the input text
/// with the default separator (`'_'`), default decimal point (`'.'`),
/// the default exponent indicators (`"e-", "e+", "e", "E-", "E+", "E"`)
/// and no accumulators.
/// Return how many bytes are digested and the float point literal data.
/// # Caveat
/// If the matched result is exponent part only, the total digested length
/// (the `return.0`) will be set to `0`.
/// E.g. if the exponent indicator is `e` then
/// `e10` won't be treated as a valid float literal.
///
/// Besides, each part (the integer part, the fraction part and the exponent part)
/// will be matched using [`decimal_integer_literal_body_with_options`]
/// so its caveat also applies here.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{float_literal_body};
/// let (digested, data) = float_literal_body(
///   "1_23.4_56e-7_8z",
/// );
/// assert_eq!(digested, 14); // total digested bytes
/// let (integer_digested, integer_data) = data.integer;
/// assert_eq!(integer_digested, 4);
/// let (fraction_digested, fraction_data) = data.fraction.unwrap();
/// assert_eq!(fraction_digested, 5); // including the `.`
/// let (exponent_digested, exponent_data) = data.exponent.unwrap();
/// assert_eq!(exponent_digested, 5); // including the `e-`
/// assert_eq!(exponent_data.indicator_len, 2);
/// ```
pub fn float_literal_body(rest: &str) -> (usize, FloatLiteralData<(), (), (), ()>) {
  float_literal_body_with_options(
    rest,
    FloatLiteralOptions::default()
      .default_separator()
      .default_fraction()
      .default_exponent(),
  )
}

/// Try to match a float point literal in the rest of the input text
/// with the given [`FloatLiteralOptions`].
/// Return how many bytes are digested and the float point literal data.
/// # Caveat
/// If the matched result is exponent part only, the total digested length
/// (the `return.0`) will be set to `0`.
/// E.g. if the exponent indicator is `e` then
/// `e10` won't be treated as a valid float literal.
///
/// Besides, each part (the integer part, the fraction part and the exponent part)
/// will be matched using [`decimal_integer_literal_body_with_options`]
/// so its caveat also applies here.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{float_literal_body_with};
/// let (digested, data) = float_literal_body_with(
///   "1_23.4_56e-7_8z",
///   |o| o
///     // enable numeric separator using `_`
///     // and collect their indexes into a vector
///     .separator_with(|s| s.ch('_').acc_to_vec())
///     // collect the value of the integer part
///     // into a string
///     .integer_to_string()
///     // enable fraction part using `.` as the decimal point
///     // and collect the value to a string
///     .fraction_with(|o| o.point('.').acc_to_string())
///     // enable exponent part using the default exponent indicators
///     // and collect the value to a string
///     .exponent_with(|o| o.acc_to_string())
/// );
/// assert_eq!(digested, 14); // total digested bytes
/// let (integer_digested, integer_data) = data.integer;
/// assert_eq!(integer_digested, 4);
/// assert_eq!(integer_data.value, "123".to_string());
/// assert_eq!(integer_data.separators, vec![1]);
/// let (fraction_digested, fraction_data) = data.fraction.unwrap();
/// assert_eq!(fraction_digested, 5); // including the `.`
/// assert_eq!(fraction_data.value, "456".to_string());
/// assert_eq!(fraction_data.separators, vec![1]); // index in the fraction body
/// let (exponent_digested, exponent_data) = data.exponent.unwrap();
/// assert_eq!(exponent_digested, 5); // including the `e-`
/// assert_eq!(exponent_data.indicator_len, 2);
/// assert_eq!(exponent_data.base.value, "78".to_string());
/// assert_eq!(exponent_data.base.separators, vec![1]); // index in the exponent body
/// ```
pub fn float_literal_body_with<
  SepAcc: NumericSeparatorAccumulator,
  IntAcc: Accumulator<char>,
  FracAcc: Accumulator<char>,
  ExpAcc: Accumulator<char>,
>(
  rest: &str,
  options_builder: impl FnOnce(
    FloatLiteralOptions<
      MockNumericSeparatorAccumulator,
      MockAccumulator,
      MockAccumulator,
      MockAccumulator,
    >,
  ) -> FloatLiteralOptions<SepAcc, IntAcc, FracAcc, ExpAcc>,
) -> (
  usize,
  FloatLiteralData<SepAcc::Target, IntAcc::Target, FracAcc::Target, ExpAcc::Target>,
) {
  float_literal_body_with_options(rest, options_builder(FloatLiteralOptions::default()))
}

/// Try to match a float point literal in the rest of the input text
/// with the given [`FloatLiteralOptions`].
/// Return how many bytes are digested and the float point literal data.
/// # Caveat
/// If the matched result is exponent part only, the total digested length
/// (the `return.0`) will be set to `0`.
/// E.g. if the exponent indicator is `e` then
/// `e10` won't be treated as a valid float literal.
///
/// Besides, each part (the integer part, the fraction part and the exponent part)
/// will be matched using [`decimal_integer_literal_body_with_options`]
/// so its caveat also applies here.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{float_literal_body_with_options, FloatLiteralOptions};
/// let (digested, data) = float_literal_body_with_options(
///   "1_23.4_56e-7_8z",
///   FloatLiteralOptions::default()
///     // enable numeric separator using `_`
///     // and collect their indexes into a vector
///     .separator_with(|s| s.ch('_').acc_to_vec())
///     // collect the value of the integer part
///     // into a string
///     .integer_to_string()
///     // enable fraction part using `.` as the decimal point
///     // and collect the value to a string
///     .fraction_with(|o| o.point('.').acc_to_string())
///     // enable exponent part using the default exponent indicators
///     // and collect the value to a string
///     .exponent_with(|o| o.acc_to_string())
/// );
/// assert_eq!(digested, 14); // total digested bytes
/// let (integer_digested, integer_data) = data.integer;
/// assert_eq!(integer_digested, 4);
/// assert_eq!(integer_data.value, "123".to_string());
/// assert_eq!(integer_data.separators, vec![1]);
/// let (fraction_digested, fraction_data) = data.fraction.unwrap();
/// assert_eq!(fraction_digested, 5); // including the `.`
/// assert_eq!(fraction_data.value, "456".to_string());
/// assert_eq!(fraction_data.separators, vec![1]); // index in the fraction body
/// let (exponent_digested, exponent_data) = data.exponent.unwrap();
/// assert_eq!(exponent_digested, 5); // including the `e-`
/// assert_eq!(exponent_data.indicator_len, 2);
/// assert_eq!(exponent_data.base.value, "78".to_string());
/// assert_eq!(exponent_data.base.separators, vec![1]); // index in the exponent body
/// ```
pub fn float_literal_body_with_options<
  SepAcc: NumericSeparatorAccumulator,
  IntAcc: Accumulator<char>,
  FracAcc: Accumulator<char>,
  ExpAcc: Accumulator<char>,
>(
  rest: &str,
  options: FloatLiteralOptions<SepAcc, IntAcc, FracAcc, ExpAcc>,
) -> (
  usize,
  FloatLiteralData<SepAcc::Target, IntAcc::Target, FracAcc::Target, ExpAcc::Target>,
) {
  let mut total_digested = 0;

  // first, parse the integer part
  let (integer_digested, integer_data) = decimal_integer_literal_body_with_options(
    rest,
    IntegerLiteralBodyOptions {
      separator: options.separator.clone(),
      value: options.integer,
    },
  );
  total_digested += integer_digested;
  // integer part may be empty, which is acceptable,
  // e.g. in some languages like JavaScript, ".5" is a valid float literal

  // next, parse the optional fraction part
  let fraction_part = options.fraction.and_then(|fraction| {
    let rest = &rest[total_digested..];
    rest.starts_with(fraction.point).then(|| {
      let (body_digested, data) = decimal_integer_literal_body_with_options(
        &rest[fraction.point.len_utf8()..],
        IntegerLiteralBodyOptions {
          separator: options.separator.clone(),
          value: fraction.acc,
        },
      );
      let fraction_digested = body_digested + fraction.point.len_utf8();
      total_digested += fraction_digested;

      (fraction_digested, data)
    })
  });

  // finally, parse the optional exponent part
  let exponent_part = options.exponent.and_then(|exponent| {
    let rest = &rest[total_digested..];
    rest.chars().next().and_then(|c| {
      exponent.indicator_heads().contains(&c).then(|| {
        let indicator = exponent
          .indicators
          .iter()
          .find(|indicator| rest.starts_with(*indicator))
          .unwrap();

        let (body_digested, data) = decimal_integer_literal_body_with_options(
          &rest[indicator.len()..],
          IntegerLiteralBodyOptions {
            separator: options.separator,
            value: exponent.acc,
          },
        );
        let exponent_digested = body_digested + indicator.len();
        total_digested += exponent_digested;

        (
          exponent_digested,
          FloatExponentData {
            indicator_len: indicator.len(),
            base: data,
          },
        )
      })
    })
  });

  // if only exponent part is present, set total digested to 0
  if integer_digested == 0 && fraction_part.is_none() {
    total_digested = 0;
  }

  (
    total_digested,
    FloatLiteralData {
      integer: (integer_digested, integer_data),
      fraction: fraction_part,
      exponent: exponent_part,
    },
  )
}

/// Create an [`Action`] that tries to match the float literal
/// in the rest of the input text
/// with the default separator (`'_'`), default decimal point (`'.'`),
/// the default exponent indicators (`"e-", "e+", "e", "E-", "E+", "E"`)
/// and no accumulators.
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Caveat
/// If the matched content is exponent only, the action will reject it.
///
/// Besides, each part (the integer part, the fraction part and the exponent part)
/// will be matched using [`decimal_integer_literal_body_with_options`]
/// so its caveat also applies here.
pub fn float_literal<ActionState, ErrorType>(
) -> Action<MockTokenKind<FloatLiteralData<(), (), (), ()>>, ActionState, ErrorType> {
  float_literal_with_options(
    FloatLiteralOptions::default()
      .default_separator()
      .default_fraction()
      .default_exponent(),
  )
}

/// Create an [`Action`] that tries to match the float literal
/// in the rest of the input text
/// with the given [`FloatLiteralOptions`].
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Caveat
/// If the matched content is exponent only, the action will reject it.
///
/// Besides, each part (the integer part, the fraction part and the exponent part)
/// will be matched using [`decimal_integer_literal_body_with_options`]
/// so its caveat also applies here.
pub fn float_literal_with<
  ActionState,
  ErrorType,
  SepAcc: NumericSeparatorAccumulator + 'static,
  IntAcc: Accumulator<char> + 'static,
  FracAcc: Accumulator<char> + 'static,
  ExpAcc: Accumulator<char> + 'static,
>(
  options_builder: impl FnOnce(
    FloatLiteralOptions<
      MockNumericSeparatorAccumulator,
      MockAccumulator,
      MockAccumulator,
      MockAccumulator,
    >,
  ) -> FloatLiteralOptions<SepAcc, IntAcc, FracAcc, ExpAcc>,
) -> Action<
  MockTokenKind<FloatLiteralData<SepAcc::Target, IntAcc::Target, FracAcc::Target, ExpAcc::Target>>,
  ActionState,
  ErrorType,
> {
  float_literal_with_options(options_builder(FloatLiteralOptions::default()))
}

/// Create an [`Action`] that tries to match the float literal
/// in the rest of the input text
/// with the given [`FloatLiteralOptions`].
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Caveat
/// If the matched content is exponent only, the action will reject it.
///
/// Besides, each part (the integer part, the fraction part and the exponent part)
/// will be matched using [`decimal_integer_literal_body_with_options`]
/// so its caveat also applies here.
pub fn float_literal_with_options<
  ActionState,
  ErrorType,
  SepAcc: NumericSeparatorAccumulator + 'static,
  IntAcc: Accumulator<char> + 'static,
  FracAcc: Accumulator<char> + 'static,
  ExpAcc: Accumulator<char> + 'static,
>(
  options: FloatLiteralOptions<SepAcc, IntAcc, FracAcc, ExpAcc>,
) -> Action<
  MockTokenKind<FloatLiteralData<SepAcc::Target, IntAcc::Target, FracAcc::Target, ExpAcc::Target>>,
  ActionState,
  ErrorType,
> {
  // head for integer part
  let mut heads = HashSet::from(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);

  // if fraction part is enabled, add the decimal point to the head
  if let Some(fraction) = options.fraction.as_ref() {
    heads.insert(fraction.point);
  }

  // don't add exponent indicators to the head
  // because we don't allow exponent part to be the only part

  simple_with_data(move |input| {
    let res = float_literal_body_with_options(&input.rest(), options.clone());

    if res.0 == 0 {
      return None;
    }

    Some(res)
  })
  .unchecked_head_in(heads)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{ActionInput, HeadMatcher};

  fn assert_float_literal_body(
    (digested, data): (usize, FloatLiteralData<Vec<usize>, String, String, String>),
    integer: (usize, &str, Vec<usize>),
    fraction: Option<(usize, &str, Vec<usize>)>,
    exponent: Option<(usize, &str, usize, Vec<usize>)>,
  ) {
    if integer.0 == 0 && fraction.is_none() {
      assert_eq!(digested, 0);
    } else {
      assert_eq!(
        digested,
        integer.0
          + fraction.as_ref().map(|f| f.0).unwrap_or_default()
          + exponent.as_ref().map(|f| f.0).unwrap_or_default()
      );
    }
    assert_eq!(data.integer.0, integer.0);
    assert_eq!(data.integer.1.value, integer.1);
    assert_eq!(data.integer.1.separators, integer.2);

    if let Some(fraction) = fraction {
      assert_eq!(data.fraction.as_ref().unwrap().0, fraction.0);
      assert_eq!(data.fraction.as_ref().unwrap().1.value, fraction.1);
      assert_eq!(data.fraction.as_ref().unwrap().1.separators, fraction.2);
    }

    if let Some(exponent) = exponent {
      assert_eq!(data.exponent.as_ref().unwrap().0, exponent.0);
      assert_eq!(data.exponent.as_ref().unwrap().1.base.value, exponent.1);
      assert_eq!(data.exponent.as_ref().unwrap().1.indicator_len, exponent.2);
      assert_eq!(
        data.exponent.as_ref().unwrap().1.base.separators,
        exponent.3
      );
    }
  }

  #[test]
  fn test_float_literal_body_with_options() {
    let options = FloatLiteralOptions::default()
      .separator_with(|s| s.acc_to_vec())
      .integer_to_string()
      .fraction_with(|o| o.acc_to_string())
      .exponent_with(|o| o.acc_to_string());

    // invalid start
    assert_float_literal_body(
      float_literal_body_with_options("z", options.clone()),
      (0, "", vec![]),
      None,
      None,
    );

    // integer only
    assert_float_literal_body(
      float_literal_body_with_options("12_3z", options.clone()),
      (4, "123", vec![2]),
      None,
      None,
    );

    // integer and fraction
    assert_float_literal_body(
      float_literal_body_with_options("12_3.4_5z", options.clone()),
      (4, "123", vec![2]),
      Some((4, "45", vec![1])),
      None,
    );

    // integer and exponent
    assert_float_literal_body(
      float_literal_body_with_options("12_3e4_5z", options.clone()),
      (4, "123", vec![2]),
      None,
      Some((4, "45", 1, vec![1])),
    );

    // fraction only
    assert_float_literal_body(
      float_literal_body_with_options(".12_3z", options.clone()),
      (0, "", vec![]),
      Some((5, "123", vec![2])),
      None,
    );

    // fraction and exponent
    assert_float_literal_body(
      float_literal_body_with_options(".12_3e4_5z", options.clone()),
      (0, "", vec![]),
      Some((5, "123", vec![2])),
      Some((4, "45", 1, vec![1])),
    );

    // exponent only
    assert_float_literal_body(
      float_literal_body_with_options("e1_0z", options.clone()),
      (0, "", vec![]),
      None,
      Some((4, "10", 1, vec![1])),
    );
    assert_float_literal_body(
      float_literal_body_with_options("e-1_0z", options.clone()),
      (0, "", vec![]),
      None,
      Some((5, "10", 2, vec![1])),
    );
    assert_float_literal_body(
      float_literal_body_with_options("e+1_0z", options.clone()),
      (0, "", vec![]),
      None,
      Some((5, "10", 2, vec![1])),
    );
    assert_float_literal_body(
      float_literal_body_with_options("E1_0z", options.clone()),
      (0, "", vec![]),
      None,
      Some((4, "10", 1, vec![1])),
    );
    assert_float_literal_body(
      float_literal_body_with_options("E-1_0z", options.clone()),
      (0, "", vec![]),
      None,
      Some((5, "10", 2, vec![1])),
    );
    assert_float_literal_body(
      float_literal_body_with_options("E+1_0z", options.clone()),
      (0, "", vec![]),
      None,
      Some((5, "10", 2, vec![1])),
    );

    // full
    assert_float_literal_body(
      float_literal_body_with_options("1_23.4_56e-7_8z", options.clone()),
      (4, "123", vec![1]),
      Some((5, "456", vec![1])),
      Some((5, "78", 2, vec![1])),
    );
  }

  #[test]
  fn test_float_literal_action_head_matcher() {
    // default
    let action: Action<_> = float_literal_with(|o| o);
    assert!(
      matches!(action.head_matcher(), Some(HeadMatcher::OneOf(set)) if set.len() == 10 && set.contains(&'0') && set.contains(&'9'))
    );

    // enable fraction
    let action: Action<_> = float_literal_with(|o| o.default_fraction());
    assert!(
      matches!(action.head_matcher(), Some(HeadMatcher::OneOf(set)) if set.len() == 11&& set.contains(&'0') && set.contains(&'9') && set.contains(&'.'))
    );

    // enable exponent
    let action: Action<_> = float_literal_with(|o| o.default_exponent());
    assert!(
      matches!(action.head_matcher(), Some(HeadMatcher::OneOf(set)) if set.len() == 10&& set.contains(&'0') && set.contains(&'9'))
    );

    // enable fraction and exponent
    let action: Action<_> = float_literal_with(|o| o.default_fraction().default_exponent());
    assert!(
      matches!(action.head_matcher(), Some(HeadMatcher::OneOf(set)) if set.len() == 11&& set.contains(&'0') && set.contains(&'9') && set.contains(&'.'))
    );
  }

  fn assert_reject<T>(action: &Action<T>, s: &str) {
    assert!(action
      .exec(&mut ActionInput::new(s, 0, &mut ()).unwrap())
      .is_none())
  }

  fn assert_accept(
    action: &Action<MockTokenKind<FloatLiteralData<Vec<usize>, String, String, String>>>,
    s: &str,
    integer: (usize, &str, Vec<usize>),
    fraction: Option<(usize, &str, Vec<usize>)>,
    exponent: Option<(usize, &str, usize, Vec<usize>)>,
  ) {
    let output = action
      .exec(&mut ActionInput::new(s, 0, &mut ()).unwrap())
      .unwrap();
    assert_float_literal_body(
      (output.digested, output.kind.data),
      integer,
      fraction,
      exponent,
    );
  }

  #[test]
  fn test_float_literal_action_with_options() {
    let options = FloatLiteralOptions::default()
      .separator_with(|s| s.acc_to_vec())
      .integer_to_string()
      .fraction_with(|o| o.acc_to_string())
      .exponent_with(|o| o.acc_to_string());
    let action: Action<_> = float_literal_with_options(options);

    // reject invalid start
    assert_reject(&action, "z");
    // reject exponent only
    assert_reject(&action, "e10");

    // full
    assert_accept(
      &action,
      "1_23.4_56e-7_8z",
      (4, "123", vec![1]),
      Some((5, "456", vec![1])),
      Some((5, "78", 2, vec![1])),
    );
  }
}
