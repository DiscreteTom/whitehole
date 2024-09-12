use super::lookup::Lookup;

/// A lookup table wrapper that won't allocate for the first `n` values to save memory.
/// This will record the offset `n` and prevent access to the first `n` values.
/// This is useful if your lookup table is sparse and starts far from 0.
#[derive(Debug, Clone)]
pub(crate) struct OffsetLookupTable<Table> {
  offset: usize,
  table: Table,
}

impl<Table> OffsetLookupTable<Table> {
  /// Create a new instance with the given offset and table.
  #[inline]
  pub const fn new(offset: usize, table: Table) -> Self {
    Self { offset, table }
  }
}

impl<Table: Lookup> Lookup for OffsetLookupTable<Table> {
  type Value = Table::Value;

  #[inline]
  fn get(&self, key: usize) -> Option<&Self::Value> {
    // check key first to prevent underflow
    if key < self.offset {
      None
    } else {
      self.table.get(key - self.offset)
    }
  }

  #[inline]
  fn len(&self) -> usize {
    // the first `n` values are not accessible but still counted
    self.offset + self.table.len()
  }

  #[inline]
  unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut Table::Value {
    debug_assert!(key >= self.offset);
    self.table.get_unchecked_mut(key - self.offset)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::lookup::option::OptionLookupTable;

  #[test]
  fn test_offset_lookup_table() {
    let option = OptionLookupTable::with_keys_init([0, 2].iter().map(|i| *i), || 0);
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
