use super::lookup::Lookup;
use std::fmt::{self, Debug};

/// A lookup table that not all keys are used.
#[derive(Clone)]
pub(crate) struct OptionLookupTable<V> {
  data: Vec<Option<V>>,
}

impl<V: Debug> Debug for OptionLookupTable<V> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // format as a map instead of a list
    f.debug_map()
      .entries(
        self
          .data
          .iter()
          .enumerate()
          .filter_map(|(i, v)| v.as_ref().map(|v| (i, v))),
      )
      .finish()
  }
}

impl<V> OptionLookupTable<V> {
  /// Create a new instance with the given size.
  /// Init all values to [`None`].
  pub fn new(size: usize) -> Self {
    let mut data = Vec::with_capacity(size);
    data.resize_with(size, || None);
    Self { data }
  }

  /// Create a new instance with the given `keys`.
  /// `keys` can be empty, unordered or duplicated.
  /// Values are initialized with the provided `factory` if its key is present.
  /// # Design
  /// We use a key slice as the parameter, so the caller
  /// doesn't need to deduplicate keys
  /// (checking duplication in a lookup table is often more efficient).
  pub fn with_keys_complex<Item>(
    items: &[Item],
    calc_key: impl Fn(&Item) -> usize,
    factory: impl Fn() -> V,
    mut on_unique_key: impl FnMut(&Item),
  ) -> Self {
    let size = items
      .iter()
      .map(|k| calc_key(k))
      .max()
      // size = max + 1
      .map(|v| v + 1)
      // if the slice is empty, the size is 0
      .unwrap_or(0);
    let mut res = Self::new(size);

    for i in items {
      let k = calc_key(i);
      // SAFETY: `k` is guaranteed to be in the range of `0..size`.
      let d = unsafe { res.get_option_unchecked_mut(k) };
      if d.is_none() {
        *d = Some(factory());
        on_unique_key(i);
      }
    }

    res
  }

  pub fn with_keys(keys: impl Iterator<Item = usize> + Clone, factory: impl Fn() -> V) -> Self {
    let size = keys
      .clone()
      .max()
      // size = max + 1
      .map(|v| v + 1)
      // if the slice is empty, the size is 0
      .unwrap_or(0);
    let mut res = Self::new(size);

    for k in keys {
      // SAFETY: `k` is guaranteed to be in the range of `0..size`.
      let d = unsafe { res.get_option_unchecked_mut(k) };
      if d.is_none() {
        *d = Some(factory());
      }
    }

    res
  }

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  #[inline]
  pub unsafe fn get_option_unchecked_mut(&mut self, key: usize) -> &mut Option<V> {
    debug_assert!(key < self.data.len());
    self.data.get_unchecked_mut(key)
  }

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  ///
  /// [`debug_assert`] is used to check if the key is in range and valid.
  #[inline]
  pub unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut V {
    let v = self.get_option_unchecked_mut(key).as_mut();
    debug_assert!(v.is_some());
    v.unwrap_unchecked()
  }

  /// Map the values to another type and return a new instance.
  pub fn map_ref<R>(&self, mapper: impl Fn(&V) -> R) -> OptionLookupTable<R> {
    OptionLookupTable {
      data: self
        .data
        .iter()
        .map(|v| v.as_ref().map(|v| mapper(v)))
        .collect(),
    }
  }

  /// Apply the function to each value.
  pub fn for_each_value_mut(&mut self, mut f: impl FnMut(&mut V)) {
    for v in &mut self.data {
      v.as_mut().map(|v| f(v));
    }
  }
}

impl<V> Lookup for OptionLookupTable<V> {
  type Value = V;

  #[inline]
  fn get(&self, key: usize) -> Option<&Self::Value> {
    self.data.get(key).unwrap_or(&None).as_ref()
  }

  #[inline]
  fn len(&self) -> usize {
    self.data.len()
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

  #[test]
  fn test_option_lookup_table_debug() {
    let mut table = OptionLookupTable::new(3);
    unsafe {
      *table.get_option_unchecked_mut(0) = Some(1);
      *table.get_option_unchecked_mut(2) = Some(2);
    }

    assert_eq!(format!("{:?}", table), "{0: 1, 2: 2}");
  }
}
