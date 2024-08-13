pub(crate) trait Lookup {
  type Value;

  /// Return the value associated with the key.
  /// Return [`None`] if the key is not found or out of range.
  fn get(&self, key: usize) -> Option<&Self::Value>;

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut Self::Value;
}
