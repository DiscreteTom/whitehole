use crate::lexer::action::IntegerLiteralData;

pub struct FloatExponentData<Sep, ExpValue> {
  /// The byte length of the exponent indicator.
  pub indicator_len: usize,
  pub base: IntegerLiteralData<Sep, ExpValue>,
}

pub struct FloatLiteralData<Sep, IntValue, FracValue, ExpValue> {
  /// How many bytes are digested for the integer part, and the data.
  pub integer: (usize, IntegerLiteralData<Sep, IntValue>),
  /// How many bytes are digested for the fraction part
  /// (including the decimal point), and the data.
  pub fraction: Option<(usize, IntegerLiteralData<Sep, FracValue>)>,
  /// How many bytes are digested for the exponent part
  /// (including the exponent indicator), and the data.
  pub exponent: Option<(usize, FloatExponentData<Sep, ExpValue>)>,
}
