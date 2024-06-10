mod data;
mod options;

pub use data::*;
pub use options::*;

use super::{
  decimal_integer_literal_body_with_options, Accumulator, IntegerLiteralBodyOptions,
  IntegerLiteralData, MockAccumulator, NumericSeparatorAccumulator,
};
use crate::lexer::{
  action::{simple_with_data, Action},
  token::MockTokenKind,
};

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

  (
    total_digested,
    FloatLiteralData {
      integer: (integer_digested, integer_data),
      fraction: fraction_part,
      exponent: exponent_part,
    },
  )
}

// pub fn float_literal_with_options<
//   ActionState,
//   ErrorType,
//   SepAcc: Accumulator<usize> + 'static,
//   IntAcc: Accumulator<char> + 'static,
//   FracAcc: Accumulator<char> + 'static,
//   ExpAcc: Accumulator<char> + 'static,
// >(
//   options: FloatLiteralOptions<SepAcc, IntAcc, FracAcc, ExpAcc>,
// ) -> Action<
//   MockTokenKind<FloatLiteralData<SepAcc::Target, IntAcc::Target, FracAcc::Target, ExpAcc::Target>>,
//   ActionState,
//   ErrorType,
// > {
//   simple_with_data(move |input| {
//     let options = options.clone();
//     Some(float_literal_body_with_options(&input.rest(), options))
//   })
//   .unchecked_head_in([
//     '.', 'e', 'E', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
//   ])
// }
