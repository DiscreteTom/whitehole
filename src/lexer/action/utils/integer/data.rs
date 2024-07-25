#[derive(Default, Debug, Clone)]
pub struct IntegerLiteralData<Sep, Value> {
  /// The byte index of numeric separators in the integer literal body.
  /// This field is set if you enable numeric separators and provide an accumulator.
  /// See [`IntegerLiteralBodyOptions::separator`](crate::lexer::action::utils::integer::IntegerLiteralBodyOptions::separator).
  pub separators: Sep,
  /// The accumulated value of the integer literal body.
  /// This field is set if you provide an accumulator.
  /// See [`IntegerLiteralBodyOptions::value_to`](crate::lexer::action::utils::integer::IntegerLiteralBodyOptions::value_to).
  pub value: Value,
}
