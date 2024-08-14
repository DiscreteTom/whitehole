use super::lookup::Lookup;

/// A lookup table that not all keys are used.
#[derive(Debug, Clone)]
pub(crate) struct OptionLookupTable<V> {
  data: Vec<Option<V>>,
}

impl<V> OptionLookupTable<V> {
  /// Create a new instance with the given size.
  /// Init all values to [`None`].
  pub fn new(size: usize) -> Self {
    let mut data = Vec::with_capacity(size);
    data.resize_with(size, || None);
    Self { data }
  }

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  #[inline]
  pub unsafe fn get_option_unchecked_mut(&mut self, key: usize) -> &mut Option<V> {
    self.data.get_unchecked_mut(key)
  }
}

impl<V> Lookup for OptionLookupTable<V> {
  type Value = V;

  #[inline]
  fn get(&self, key: usize) -> Option<&Self::Value> {
    self.data.get(key).unwrap_or(&None).as_ref()
  }

  #[inline]
  unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut Self::Value {
    self
      .get_option_unchecked_mut(key)
      .as_mut()
      .unwrap_unchecked()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_option_lookup_table() {
    let mut table = OptionLookupTable::new(3);
    assert_eq!(table.get(0), None);
    assert_eq!(table.get(1), None);
    assert_eq!(table.get(2), None);

    unsafe {
      *table.get_option_unchecked_mut(0) = Some(1);
      *table.get_option_unchecked_mut(2) = Some(2);
    }

    assert_eq!(table.get(0), Some(&1));
    assert_eq!(table.get(1), None);
    assert_eq!(table.get(2), Some(&2));

    unsafe {
      *table.get_unchecked_mut(0) = 3;
      *table.get_unchecked_mut(2) = 4;

      assert_eq!(table.get(0), Some(&3));
      assert_eq!(table.get(2), Some(&4));
    }
  }
}
