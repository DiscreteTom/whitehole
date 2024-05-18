pub struct IntegerLiteralData<Body> {
  /// The indexes (in bytes) of separators in the rest of the input text.
  pub separators: Vec<usize>,
  pub body: Body,
}
