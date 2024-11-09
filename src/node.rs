/// See [`Node::range`].
pub type Range = std::ops::Range<usize>;

/// A kind value with a byte range.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Node<Kind> {
  /// The kind value.
  pub kind: Kind,
  /// The byte range of the node in the input text.
  /// This can be used to index the input text.
  /// # Example
  /// ```
  /// # use whitehole::Node;
  /// let node = Node {
  ///   kind: (),
  ///   range: 0..5,
  /// };
  /// // index a string with the range
  /// assert_eq!(&"0123456"[node.range], "01234");
  pub range: Range,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_node() {
    let node = Node {
      kind: (),
      range: 0..5, // ensure we can create the range with the range syntax
    };

    // ensure the range can be used to index a string
    assert_eq!(&"0123456"[node.range], "01234");
  }
}
