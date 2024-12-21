/// See [`WithRange::range`].
pub type Range = std::ops::Range<usize>;

/// Associate a data with a byte range.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WithRange<Data> {
  /// The original data.
  pub data: Data,
  /// A byte range.
  /// This can be used to index a string.
  /// # Example
  /// ```
  /// # use whitehole::range::WithRange;
  /// let value = WithRange {
  ///   data: (),
  ///   range: 0..5,
  /// };
  /// // index a string with the range
  /// assert_eq!(&"0123456"[value.range], "01234");
  pub range: Range,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_node() {
    let value = WithRange {
      data: (),
      range: 0..5, // ensure we can create the range with the range syntax
    };

    // ensure the range can be used to index a string
    assert_eq!(&"0123456"[value.range], "01234");
  }
}
