// TODO: only available in feature "json"

use super::{
  chars_in_str, exact_chars, float_literal_body_with_options, map, unicode_with, Accumulator,
  FloatLiteralData, FloatLiteralOptions, HexEscapeError, PartialStringBody, PartialStringBodyValue,
  StringBodyOptions,
};
use crate::lexer::{
  action::{simple_with_data, Action},
  token::MockTokenKind,
};

pub fn whitespaces<ActionState, ErrorType>() -> Action<MockTokenKind<()>, ActionState, ErrorType> {
  chars_in_str("\x20\x0a\x0d\x09")
}

pub fn boundaries<ActionState, ErrorType>() -> Vec<Action<MockTokenKind<()>, ActionState, ErrorType>>
{
  exact_chars("{},:[]")
}

pub struct StringOptions<BodyAcc, CustomError> {
  /// The accumulator for the string body.
  pub acc: BodyAcc,
  /// The error mapper for hex escape errors.
  pub error_mapper: Box<dyn Fn(HexEscapeError) -> CustomError>,
}

impl StringOptions<(), HexEscapeError> {
  pub fn new() -> Self {
    Self {
      acc: (),
      error_mapper: Box::new(|e| e),
    }
  }
}

impl<BodyAcc, CustomError> StringOptions<BodyAcc, CustomError> {
  /// Set the accumulator for the string body.
  pub fn acc<NewAcc>(self, acc: NewAcc) -> StringOptions<NewAcc, CustomError> {
    StringOptions {
      acc,
      error_mapper: self.error_mapper,
    }
  }

  /// Set the error mapper for hex escape errors.
  pub fn error<NewError>(
    self,
    error_mapper: impl Fn(HexEscapeError) -> NewError + 'static,
  ) -> StringOptions<BodyAcc, NewError> {
    StringOptions {
      acc: self.acc,
      error_mapper: Box::new(error_mapper),
    }
  }
}

// TODO: comments
pub fn string<
  ActionState,
  ErrorType,
  Value: PartialStringBodyValue + 'static,
  CustomError: 'static,
  BodyAcc: Accumulator<PartialStringBody<Value, CustomError>> + Clone,
>(
  options: StringOptions<BodyAcc, CustomError>,
) -> Action<MockTokenKind<BodyAcc>, ActionState, ErrorType> {
  super::string(
    "\"",
    StringBodyOptions::default()
      .chars(|c| match c {
        '\x20'..='\u{10ffff}' => c != '"' && c != '\\' && c != '\r' && c != '\n',
        _ => false,
      })
      .escape(
        '\\',
        [
          map([
            ('"', '"'),
            ('\\', '\\'),
            ('/', '/'),
            ('b', '\x08'),
            ('f', '\x0c'),
            ('n', '\n'),
            ('r', '\r'),
            ('t', '\t'),
            ('0', '\0'),
          ]),
          unicode_with(|o| o.error(options.error_mapper)),
        ],
      )
      .close('"')
      .singleline()
      .acc(options.acc),
  )
}

pub fn string_with<
  ActionState,
  ErrorType,
  Value: PartialStringBodyValue + 'static,
  CustomError: 'static,
  BodyAcc: Accumulator<PartialStringBody<Value, CustomError>> + Clone,
>(
  options_builder: impl FnOnce(StringOptions<(), HexEscapeError>) -> StringOptions<BodyAcc, CustomError>,
) -> Action<MockTokenKind<BodyAcc>, ActionState, ErrorType> {
  string(options_builder(StringOptions::new()))
}

#[derive(Default, Debug, Clone)]
pub struct NumberOptions<SepAcc, IntAcc, FracAcc, ExpAcc> {
  /// See [`Self::separator`].
  pub separator: SepAcc,
  /// See [`Self::integer`].
  pub integer: IntAcc,
  /// See [`Self::fraction`].
  pub fraction: FracAcc,
  /// See [`Self::exponent`].
  pub exponent: ExpAcc,
}

impl NumberOptions<(), (), (), ()> {
  /// Create a new [`Self`] with no accumulators.
  pub fn new() -> Self {
    NumberOptions {
      separator: (),
      integer: (),
      fraction: (),
      exponent: (),
    }
  }
}

impl<SepAcc, IntAcc, FracAcc, ExpAcc> NumberOptions<SepAcc, IntAcc, FracAcc, ExpAcc> {
  /// Set the accumulator for the separator part.
  pub fn separator<NewSepAcc>(
    self,
    acc: NewSepAcc,
  ) -> NumberOptions<NewSepAcc, IntAcc, FracAcc, ExpAcc> {
    NumberOptions {
      separator: acc,
      integer: self.integer,
      fraction: self.fraction,
      exponent: self.exponent,
    }
  }

  /// Set the accumulator for the integer part.
  pub fn integer<NewIntAcc>(
    self,
    acc: NewIntAcc,
  ) -> NumberOptions<SepAcc, NewIntAcc, FracAcc, ExpAcc> {
    NumberOptions {
      separator: self.separator,
      integer: acc,
      fraction: self.fraction,
      exponent: self.exponent,
    }
  }

  /// Set the accumulator for the fractional part.
  pub fn fraction<NewFracAcc>(
    self,
    acc: NewFracAcc,
  ) -> NumberOptions<SepAcc, IntAcc, NewFracAcc, ExpAcc> {
    NumberOptions {
      separator: self.separator,
      integer: self.integer,
      fraction: acc,
      exponent: self.exponent,
    }
  }

  /// Set the accumulator for the exponent part.
  pub fn exponent<NewExpAcc>(
    self,
    acc: NewExpAcc,
  ) -> NumberOptions<SepAcc, IntAcc, FracAcc, NewExpAcc> {
    NumberOptions {
      separator: self.separator,
      integer: self.integer,
      fraction: self.fraction,
      exponent: acc,
    }
  }
}

pub fn number<
  ActionState,
  ErrorType,
  SepAcc: Accumulator<usize> + Clone,
  IntAcc: Accumulator<char> + Clone,
  FracAcc: Accumulator<char> + Clone,
  ExpAcc: Accumulator<char> + Clone,
>(
  options: NumberOptions<SepAcc, IntAcc, FracAcc, ExpAcc>,
) -> Action<MockTokenKind<FloatLiteralData<SepAcc, IntAcc, FracAcc, ExpAcc>>, ActionState, ErrorType>
{
  let options = FloatLiteralOptions::default()
    .separator_with(|o| o.acc(options.separator))
    .integer(options.integer)
    .fraction_with(|o| o.acc(options.fraction))
    .exponent_with(|o| o.acc(options.exponent));

  simple_with_data(
    move |input: &crate::lexer::action::ActionInput<ActionState>| {
      let mut digested = 0;
      if input.next() == '-' {
        digested += 1;
      }

      let mut res = float_literal_body_with_options(&input.rest()[digested..], options.clone());
      res.0 += digested;

      if res.0 == 0 {
        return None;
      }

      Some(res)
    },
  )
  .unchecked_head_in(['-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'])
}

pub fn number_with<
  ActionState,
  ErrorType,
  SepAcc: Accumulator<usize> + Clone,
  IntAcc: Accumulator<char> + Clone,
  FracAcc: Accumulator<char> + Clone,
  ExpAcc: Accumulator<char> + Clone,
>(
  options_builder: impl FnOnce(
    NumberOptions<(), (), (), ()>,
  ) -> NumberOptions<SepAcc, IntAcc, FracAcc, ExpAcc>,
) -> Action<MockTokenKind<FloatLiteralData<SepAcc, IntAcc, FracAcc, ExpAcc>>, ActionState, ErrorType>
{
  number(options_builder(NumberOptions::new()))
}

