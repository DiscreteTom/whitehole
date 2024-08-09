use super::Lookup;

/// A lookup table wrapper that treat the first `n` values as empty.
/// This will record the offset `n` and prevent access to the first `n` values.
#[derive(Debug, Clone)]
pub(crate) struct OffsetLookupTable<Table> {
  offset: usize,
  table: Table,
}

impl<Table> OffsetLookupTable<Table> {
  pub fn new(offset: usize, table: Table) -> Self {
    Self { offset, table }
  }
}

impl<Table: Lookup> Lookup for OffsetLookupTable<Table> {
  type Value = Table::Value;

  #[inline]
  fn get(&self, key: usize) -> Option<&Self::Value> {
    if key < self.offset {
      None
    } else {
      self.table.get(key - self.offset)
    }
  }

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  /// # Panics
  /// Panics if the key is smaller than [`Self::offset`].
  #[inline]
  unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut Self::Value {
    self.table.get_unchecked_mut(key - self.offset)
  }
}
