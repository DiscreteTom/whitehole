/// A trait for a lookup table.
pub(crate) trait Lookup {
  type Value;

  /// Return the value associated with the key.
  /// Return [`None`] if the key is not found or out of range.
  fn get(&self, key: usize) -> Option<&Self::Value>;

  /// Return the length of the table.
  /// This may not be the allocated size of the table.
  /// If a `key` is equal to or greater than the length,
  /// [`Lookup::get`] will always return [`None`].
  fn len(&self) -> usize;
}
