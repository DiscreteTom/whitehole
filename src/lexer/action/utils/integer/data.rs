pub struct IntegerLiteralData<Sep, Value> {
  /// The byte index of numeric separators in the integer literal body.
  pub separators: Sep,
  pub value: Value,
}
