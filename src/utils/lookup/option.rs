use super::lookup::Lookup;
use std::fmt::{self, Debug};

/// A lookup table that not all keys are used.
#[derive(Clone)]
pub(crate) struct OptionLookupTable<V> {
  data: Vec<Option<V>>,
}

impl<V: Debug> Debug for OptionLookupTable<V> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
  pub fn with_keys(keys: &[usize], factory: impl Fn() -> V) -> Self {
    // ensure the `keys` are not empty
    if keys.is_empty() {
      return Self::new(0);
    }

    // SAFETY: `keys` is not empty, so the `max` is safe to unwrap.
    let max = *unsafe { keys.iter().max().unwrap_unchecked() };
    let size = max + 1;
    let mut res = Self::new(size);

    for k in keys {
      // SAFETY: `k` is guaranteed to be in the range of `0..=max`.
      let d = unsafe { res.get_option_unchecked_mut(*k) };
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
    self.data.get_unchecked_mut(key)
  }

  pub fn map<R>(&self, mapper: impl Fn(&V) -> R) -> OptionLookupTable<R> {
    OptionLookupTable {
      data: self
        .data
        .iter()
        .map(|v| v.as_ref().map(|v| mapper(v)))
        .collect(),
    }
  }

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
