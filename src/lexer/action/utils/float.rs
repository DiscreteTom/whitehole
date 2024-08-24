mod data;
mod options;

pub use data::*;
pub use options::*;

use super::{
  decimal_integer_literal_body_with_options, IntegerLiteralBodyOptions, NumericSeparatorAccumulator,
};
use crate::{
  lexer::{
    action::{simple_with_data, Action},
    token::MockTokenKind,
  },
  utils::Accumulator,
};
use std::collections::HashSet;

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
///     .separator_with(|s| s.char('_').indexes_to_vec())
///     // collect the value of the integer part
///     // into a string
///     .integral_to_string()
///     // enable fraction part using `.` as the decimal point
///     // and collect the value to a string
///     .fractional_with(|o| o.point('.').value_to_string())
///     // enable exponent part using the default exponent indicators
///     // and collect the value to a string
///     .exponent_with(|o| o.value_to_string())
/// );
/// assert_eq!(digested, 14); // total digested bytes
/// let (integer_digested, integer_data) = data.integral;
/// assert_eq!(integer_digested, 4);
/// assert_eq!(integer_data.value, "123".to_string());
/// assert_eq!(integer_data.separators, vec![1]);
/// let (fraction_digested, fraction_data) = data.fractional.unwrap();
/// assert_eq!(fraction_digested, 5); // including the `.`
/// assert_eq!(fraction_data.value, "456".to_string());
/// assert_eq!(fraction_data.separators, vec![1]); // index in the fraction body
/// let (exponent_digested, exponent_data) = data.exponent.unwrap();
/// assert_eq!(exponent_digested, 5); // including the `e-`
/// assert_eq!(exponent_data.indicator_len, 2);
/// assert_eq!(exponent_data.body.value, "78".to_string());
/// assert_eq!(exponent_data.body.separators, vec![1]); // index in the exponent body
/// ```
pub fn float_literal_body_with<
  SepAcc: NumericSeparatorAccumulator + Clone,
  IntAcc: Accumulator<char>,
  FracAcc: Accumulator<char>,
  ExpAcc: Accumulator<char>,
>(
  rest: &str,
  options_builder: impl FnOnce(
    FloatLiteralOptions<(), (), (), ()>,
  ) -> FloatLiteralOptions<SepAcc, IntAcc, FracAcc, ExpAcc>,
) -> (
  usize,
  FloatLiteralData<SepAcc::Acc, IntAcc, FracAcc, ExpAcc>,
) {
  float_literal_body_with_options(rest, options_builder(FloatLiteralOptions::new()))
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
///   FloatLiteralOptions::new()
///     // enable numeric separator using `_`
///     // and collect their indexes into a vector
///     .separator_with(|s| s.char('_').indexes_to_vec())
///     // collect the value of the integer part
///     // into a string
///     .integral_to_string()
///     // enable fraction part using `.` as the decimal point
///     // and collect the value to a string
///     .fractional_with(|o| o.point('.').value_to_string())
///     // enable exponent part using the default exponent indicators
///     // and collect the value to a string
///     .exponent_with(|o| o.value_to_string())
/// );
/// assert_eq!(digested, 14); // total digested bytes
/// let (integer_digested, integer_data) = data.integral;
/// assert_eq!(integer_digested, 4);
/// assert_eq!(integer_data.value, "123".to_string());
/// assert_eq!(integer_data.separators, vec![1]);
/// let (fraction_digested, fraction_data) = data.fractional.unwrap();
/// assert_eq!(fraction_digested, 5); // including the `.`
/// assert_eq!(fraction_data.value, "456".to_string());
/// assert_eq!(fraction_data.separators, vec![1]); // index in the fraction body
/// let (exponent_digested, exponent_data) = data.exponent.unwrap();
/// assert_eq!(exponent_digested, 5); // including the `e-`
/// assert_eq!(exponent_data.indicator_len, 2);
/// assert_eq!(exponent_data.body.value, "78".to_string());
/// assert_eq!(exponent_data.body.separators, vec![1]); // index in the exponent body
/// ```
pub fn float_literal_body_with_options<
  SepAcc: NumericSeparatorAccumulator + Clone,
  IntAcc: Accumulator<char>,
  FracAcc: Accumulator<char>,
  ExpAcc: Accumulator<char>,
>(
  rest: &str,
  options: FloatLiteralOptions<SepAcc, IntAcc, FracAcc, ExpAcc>,
) -> (
  usize,
  FloatLiteralData<SepAcc::Acc, IntAcc, FracAcc, ExpAcc>,
) {
  let mut total_digested = 0;

  // first, parse the integer part
  let (integer_digested, integer_data) = decimal_integer_literal_body_with_options(
    rest,
    IntegerLiteralBodyOptions {
      separator: options.separator.clone(),
      value_to: options.integral_to,
    },
  );
  total_digested += integer_digested;
  // integer part may be empty, which is acceptable,
  // e.g. in some languages like JavaScript, ".5" is a valid float literal

  // next, parse the optional fraction part
  let fraction_part = options.fractional.and_then(|fraction| {
    let rest = &rest[total_digested..];
    rest.starts_with(fraction.point).then(|| {
      let (body_digested, data) = decimal_integer_literal_body_with_options(
        &rest[fraction.point.len_utf8()..],
        IntegerLiteralBodyOptions {
          separator: options.separator.clone(),
          value_to: fraction.value_to,
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
        // find the first matched indicator
        // TODO: will regex be faster? since the regex will be compiled into a state machine
        // and match all the candidates at the same time
        let indicator = exponent
          .indicators
          .iter()
          .find(|indicator| rest.starts_with(*indicator))
          .unwrap();

        let (body_digested, data) = decimal_integer_literal_body_with_options(
          &rest[indicator.len()..],
          IntegerLiteralBodyOptions {
            separator: options.separator,
            value_to: exponent.value_to,
          },
        );
        let exponent_digested = body_digested + indicator.len();
        total_digested += exponent_digested;

        (
          exponent_digested,
          FloatExponentData {
            indicator_len: indicator.len(),
            body: data,
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
      integral: (integer_digested, integer_data),
      fractional: fraction_part,
      exponent: exponent_part,
    },
  )
}

/// Create an [`Action`] that tries to match the float literal
/// in the rest of the input text
/// with the given [`FloatLiteralOptions`].
///
/// The [`Action::head`] will be set automatically.
/// # Caveat
/// If the matched content is exponent only, the action will reject it.
///
/// Besides, each part (the integer part, the fraction part and the exponent part)
/// will be matched using [`decimal_integer_literal_body_with_options`]
/// so its caveat also applies here.
pub fn float_literal_with<
  State,
  ErrorType,
  SepAcc: NumericSeparatorAccumulator + Clone + 'static,
  IntAcc: Accumulator<char> + Clone + 'static,
  FracAcc: Accumulator<char> + Clone + 'static,
  ExpAcc: Accumulator<char> + Clone + 'static,
>(
  options_builder: impl FnOnce(
    FloatLiteralOptions<(), (), (), ()>,
  ) -> FloatLiteralOptions<SepAcc, IntAcc, FracAcc, ExpAcc>,
) -> Action<MockTokenKind<FloatLiteralData<SepAcc::Acc, IntAcc, FracAcc, ExpAcc>>, State, ErrorType>
{
  float_literal_with_options(options_builder(FloatLiteralOptions::new()))
}

/// Create an [`Action`] that tries to match the float literal
/// in the rest of the input text
/// with the given [`FloatLiteralOptions`].
///
/// The [`Action::head`] will be set automatically.
/// # Caveat
/// If the matched content is exponent only, the action will reject it.
///
/// Besides, each part (the integer part, the fraction part and the exponent part)
/// will be matched using [`decimal_integer_literal_body_with_options`]
/// so its caveat also applies here.
pub fn float_literal_with_options<
  State,
  ErrorType,
  SepAcc: NumericSeparatorAccumulator + Clone + 'static,
  IntAcc: Accumulator<char> + Clone + 'static,
  FracAcc: Accumulator<char> + Clone + 'static,
  ExpAcc: Accumulator<char> + Clone + 'static,
>(
  options: FloatLiteralOptions<SepAcc, IntAcc, FracAcc, ExpAcc>,
) -> Action<MockTokenKind<FloatLiteralData<SepAcc::Acc, IntAcc, FracAcc, ExpAcc>>, State, ErrorType>
{
  // head for integer part
  let mut heads = HashSet::from(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);

  // if fraction part is enabled, add the decimal point to the head
  if let Some(fraction) = options.fractional.as_ref() {
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
    assert_eq!(data.integral.0, integer.0);
    assert_eq!(data.integral.1.value, integer.1);
    assert_eq!(data.integral.1.separators, integer.2);

    if let Some(fraction) = fraction {
      assert_eq!(data.fractional.as_ref().unwrap().0, fraction.0);
      assert_eq!(data.fractional.as_ref().unwrap().1.value, fraction.1);
      assert_eq!(data.fractional.as_ref().unwrap().1.separators, fraction.2);
    }

    if let Some(exponent) = exponent {
      assert_eq!(data.exponent.as_ref().unwrap().0, exponent.0);
      assert_eq!(data.exponent.as_ref().unwrap().1.body.value, exponent.1);
      assert_eq!(data.exponent.as_ref().unwrap().1.indicator_len, exponent.2);
      assert_eq!(
        data.exponent.as_ref().unwrap().1.body.separators,
        exponent.3
      );
    }
  }

  #[test]
  fn test_float_literal_body_with_options() {
    let options = FloatLiteralOptions::new()
      .separator_with(|s| s.indexes_to_vec())
      .integral_to_string()
      .fractional_with(|o| o.value_to_string())
      .exponent_with(|o| o.value_to_string());

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
  fn test_float_literal_action_head() {
    // default
    let action: Action<_> = float_literal_with(|o| o);
    assert!(
      matches!(action.head(), Some(HeadMatcher::OneOf(set)) if set.len() == 10 && set.contains(&'0') && set.contains(&'9'))
    );

    // enable fraction
    let action: Action<_> = float_literal_with(|o| o.default_fractional());
    assert!(
      matches!(action.head(), Some(HeadMatcher::OneOf(set)) if set.len() == 11&& set.contains(&'0') && set.contains(&'9') && set.contains(&'.'))
    );

    // enable exponent
    let action: Action<_> = float_literal_with(|o| o.default_exponent());
    assert!(
      matches!(action.head(), Some(HeadMatcher::OneOf(set)) if set.len() == 10&& set.contains(&'0') && set.contains(&'9'))
    );

    // enable fraction and exponent
    let action: Action<_> = float_literal_with(|o| o.default_fractional().default_exponent());
    assert!(
      matches!(action.head(), Some(HeadMatcher::OneOf(set)) if set.len() == 11&& set.contains(&'0') && set.contains(&'9') && set.contains(&'.'))
    );
  }

  fn assert_reject<T>(action: &Action<T>, s: &str) {
    assert!((action.exec.raw)(&mut ActionInput::new(s, 0, &mut ()).unwrap()).is_none())
  }

  fn assert_accept(
    action: &Action<MockTokenKind<FloatLiteralData<Vec<usize>, String, String, String>>>,
    s: &str,
    integer: (usize, &str, Vec<usize>),
    fraction: Option<(usize, &str, Vec<usize>)>,
    exponent: Option<(usize, &str, usize, Vec<usize>)>,
  ) {
    let output = (action.exec.raw)(&mut ActionInput::new(s, 0, &mut ()).unwrap()).unwrap();
    assert_float_literal_body(
      (output.digested, output.binding.take().data),
      integer,
      fraction,
      exponent,
    );
  }

  #[test]
  fn test_float_literal_action_with_options() {
    let options = FloatLiteralOptions::new()
      .separator_with(|s| s.indexes_to_vec())
      .integral_to_string()
      .fractional_with(|o| o.value_to_string())
      .exponent_with(|o| o.value_to_string());
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
