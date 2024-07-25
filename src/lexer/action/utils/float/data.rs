use crate::lexer::action::IntegerLiteralData;

#[derive(Default, Debug, Clone)]
pub struct FloatExponentData<Sep, ExpValue> {
  /// The byte length of the exponent indicator.
  pub indicator_len: usize,
  /// The data of the exponent body.
  pub body: IntegerLiteralData<Sep, ExpValue>,
}

#[derive(Default, Debug, Clone)]
pub struct FloatLiteralData<Sep, IntValue, FracValue, ExpValue> {
  /// How many bytes are digested for the integral part, and the data.
  pub integral: (usize, IntegerLiteralData<Sep, IntValue>),
  /// How many bytes are digested for the fractional part
  /// (including the decimal point), and the data.
  pub fractional: Option<(usize, IntegerLiteralData<Sep, FracValue>)>,
  /// How many bytes are digested for the exponent part
  /// (including the exponent indicator), and the data.
  pub exponent: Option<(usize, FloatExponentData<Sep, ExpValue>)>,
}
