use super::lookup::Lookup;

/// A lookup table wrapper that treat the first `n` values as empty.
/// This will record the offset `n` and prevent access to the first `n` values.
#[derive(Debug, Clone)]
pub(crate) struct OffsetLookupTable<Table> {
  offset: usize,
  table: Table,
}

impl<Table> OffsetLookupTable<Table> {
  #[inline]
  pub const fn new(offset: usize, table: Table) -> Self {
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

  #[inline]
  fn len(&self) -> usize {
    self.offset + self.table.len()
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::lookup::option::OptionLookupTable;

  #[test]
  fn test_offset_lookup_table() {
    let mut option = OptionLookupTable::new(3);
    unsafe {
      *option.get_option_unchecked_mut(0) = Some(0);
      *option.get_option_unchecked_mut(2) = Some(0);
    }
    let mut table = OffsetLookupTable::new(3, option);
    assert_eq!(table.get(0), None);
    assert_eq!(table.get(1), None);
    assert_eq!(table.get(2), None);
    assert_eq!(table.get(3), Some(&0));
    assert_eq!(table.get(4), None);
    assert_eq!(table.get(5), Some(&0));

    unsafe {
      *table.get_unchecked_mut(3) = 1;
      *table.get_unchecked_mut(5) = 2;
    }

    assert_eq!(table.get(0), None);
    assert_eq!(table.get(1), None);
    assert_eq!(table.get(2), None);
    assert_eq!(table.get(3), Some(&1));
    assert_eq!(table.get(4), None);
    assert_eq!(table.get(5), Some(&2));
  }
}
