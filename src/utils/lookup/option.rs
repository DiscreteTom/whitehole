use super::Lookup;
use std::{
  fmt::{self, Debug},
  iter::{Enumerate, FlatMap},
  slice,
};

/// A lookup table that not all keys are used.
#[derive(Clone)]
pub struct OptionLookupTable<V> {
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
  #[inline]
  pub fn with_size(size: usize) -> Self {
    let mut data = Vec::with_capacity(size);
    data.resize_with(size, || None);
    Self { data }
  }

  /// Create a new instance with the given keys.
  /// `keys` can be empty, unordered or duplicated.
  /// Init all values to [`None`].
  // TODO: come up with a better name and make this public?
  #[inline]
  fn with_keys_none(keys: impl Iterator<Item = usize>) -> Self {
    Self::with_size(
      keys
        .max()
        // size = max + 1
        .map(|v| v + 1)
        // if the slice is empty, the size is 0
        .unwrap_or(0),
    )
  }

  /// Create a new instance with the given `keys`.
  /// `keys` can be empty, unordered or duplicated.
  /// Values are initialized with cloned `value` if its key is present.
  /// # Design
  /// We use a key iterator as the parameter, so the caller doesn't need to
  /// allocate a slice for the keys or deduplicate keys
  /// (checking duplication in a lookup table is often more efficient).
  #[inline]
  pub fn with_keys(keys: impl Iterator<Item = usize> + Clone, value: V) -> Self
  where
    V: Clone,
  {
    Self::with_keys_init(keys, || value.clone())
  }

  /// Create a new instance with the given `keys`.
  /// `keys` can be empty, unordered or duplicated.
  /// Values are initialized with the provided `factory` if its key is present.
  /// # Design
  /// We use a key iterator as the parameter, so the caller doesn't need to
  /// allocate a slice for the keys or deduplicate keys
  /// (checking duplication in a lookup table is often more efficient).
  #[inline]
  pub fn with_keys_init(
    keys: impl Iterator<Item = usize> + Clone,
    factory: impl FnMut() -> V,
  ) -> Self {
    let mut res = Self::with_keys_none(keys.clone());
    unsafe { res.init_unchecked_with(keys, factory) }
    res
  }

  /// Init [`None`] values with the given `keys`.
  /// `keys` can be empty, unordered or duplicated.
  /// Duplicated keys will only initialize the value at most once.
  /// If the value is already [`Some`], it will NOT be overwritten.
  /// Values are initialized with the provided `factory` if its key is present.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the keys are out of range.
  /// # Design
  /// We use a key iterator as the parameter, so the caller doesn't need to
  /// allocate a slice for the keys or deduplicate keys
  /// (checking duplication in a lookup table is often more efficient).
  #[inline]
  pub unsafe fn init_unchecked(&mut self, keys: impl Iterator<Item = usize>, value: V)
  where
    V: Clone,
  {
    self.init_unchecked_with(keys, || value.clone());
  }

  /// Init [`None`] values with the given `keys`.
  /// `keys` can be empty, unordered or duplicated.
  /// Duplicated keys will only initialize the value at most once.
  /// If the value is already [`Some`], it will NOT be overwritten.
  /// Values are initialized with the provided `factory` if its key is present.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the keys are out of range.
  /// # Design
  /// We use a key iterator as the parameter, so the caller doesn't need to
  /// allocate a slice for the keys or deduplicate keys
  /// (checking duplication in a lookup table is often more efficient).
  pub unsafe fn init_unchecked_with(
    &mut self,
    keys: impl Iterator<Item = usize>,
    mut factory: impl FnMut() -> V,
  ) {
    for k in keys {
      // SAFETY: `k` is guaranteed to be in the range of `0..size`.
      let d = self.get_option_unchecked_mut(k);
      if d.is_none() {
        *d = Some(factory());
      }
    }
  }

  /// Return an iterator over the non-[`None`] key-value pairs.
  #[inline]
  pub fn iter(&self) -> Iter<V> {
    Iter::new(self)
  }

  /// Return an iterator over the keys with non-[`None`] values.
  #[inline]
  pub fn keys(&self) -> Keys<V> {
    Keys::new(self)
  }

  /// Return an iterator over the non-[`None`] values.
  #[inline]
  pub fn values(&self) -> Values<V> {
    Values::new(self)
  }

  /// Return a mutable iterator over the non-[`None`] key-value pairs.
  #[inline]
  pub fn iter_mut(&mut self) -> IterMut<V> {
    IterMut::new(self)
  }

  /// Return a mutable iterator over the non-[`None`] values.
  #[inline]
  pub fn values_mut(&mut self) -> ValuesMut<V> {
    ValuesMut::new(self)
  }

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  ///
  /// [`debug_assert`] is used to check if the key is in range and valid.
  #[inline]
  unsafe fn get_option_unchecked_mut(&mut self, key: usize) -> &mut Option<V> {
    debug_assert!(key < self.data.len());
    self.data.get_unchecked_mut(key)
  }

  /// Create a new instance with the same size and values mapped by the provided function.
  pub fn map_to_new<R>(&self, mapper: impl Fn(&V) -> R) -> OptionLookupTable<R> {
    OptionLookupTable {
      data: self
        .data
        .iter()
        .map(|v| v.as_ref().map(|v| mapper(v)))
        .collect(),
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

  #[inline]
  unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut Self::Value {
    let v = self.get_option_unchecked_mut(key).as_mut();
    debug_assert!(v.is_some());
    v.unwrap_unchecked()
  }
}

/// See [`OptionLookupTable::iter`].
#[derive(Clone, Debug)]
pub struct Iter<'a, V> {
  iter: FlatMap<
    Enumerate<slice::Iter<'a, Option<V>>>,
    Option<(usize, &'a V)>,
    fn((usize, &Option<V>)) -> Option<(usize, &V)>,
  >,
}

impl<'a, V> Iter<'a, V> {
  #[inline]
  fn new(table: &'a OptionLookupTable<V>) -> Self {
    fn mapper<V>((k, v): (usize, &Option<V>)) -> Option<(usize, &V)> {
      v.as_ref().map(|v| (k, v))
    }
    Self {
      iter: table.data.iter().enumerate().flat_map(mapper),
    }
  }
}

impl<'a, V> Iterator for Iter<'a, V> {
  type Item = (usize, &'a V);

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}

/// See [`OptionLookupTable::keys`].
#[derive(Clone, Debug)]
pub struct Keys<'a, V> {
  iter: Iter<'a, V>,
}

impl<'a, V> Keys<'a, V> {
  #[inline]
  fn new(table: &'a OptionLookupTable<V>) -> Self {
    Self {
      iter: Iter::new(table),
    }
  }
}

impl<'a, V> Iterator for Keys<'a, V> {
  type Item = usize;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next().map(|(k, _)| k)
  }
}

/// See [`OptionLookupTable::values`].
#[derive(Clone, Debug)]
pub struct Values<'a, V> {
  iter: Iter<'a, V>,
}

impl<'a, V> Values<'a, V> {
  #[inline]
  fn new(table: &'a OptionLookupTable<V>) -> Self {
    Self {
      iter: Iter::new(table),
    }
  }
}

impl<'a, V> Iterator for Values<'a, V> {
  type Item = &'a V;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next().map(|(_, v)| v)
  }
}

/// See [`OptionLookupTable::iter_mut`].
#[derive(Debug)]
pub struct IterMut<'a, V> {
  iter: FlatMap<
    Enumerate<slice::IterMut<'a, Option<V>>>,
    Option<(usize, &'a mut V)>,
    fn((usize, &mut Option<V>)) -> Option<(usize, &mut V)>,
  >,
}

impl<'a, V> IterMut<'a, V> {
  #[inline]
  fn new(table: &'a mut OptionLookupTable<V>) -> Self {
    fn mapper<V>((k, v): (usize, &mut Option<V>)) -> Option<(usize, &mut V)> {
      v.as_mut().map(|v| (k, v))
    }
    Self {
      iter: table.data.iter_mut().enumerate().flat_map(mapper),
    }
  }
}

impl<'a, V> Iterator for IterMut<'a, V> {
  type Item = (usize, &'a mut V);

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
}

/// See [`OptionLookupTable::values_mut`].
#[derive(Debug)]
pub struct ValuesMut<'a, V> {
  iter: IterMut<'a, V>,
}

impl<'a, V> ValuesMut<'a, V> {
  #[inline]
  fn new(table: &'a mut OptionLookupTable<V>) -> Self {
    Self {
      iter: IterMut::new(table),
    }
  }
}

impl<'a, V> Iterator for ValuesMut<'a, V> {
  type Item = &'a mut V;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next().map(|(_, v)| v)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_option_lookup_table() {
    let mut table = OptionLookupTable::with_size(3);
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
    let mut table = OptionLookupTable::with_size(3);
    unsafe {
      *table.get_option_unchecked_mut(0) = Some(1);
      *table.get_option_unchecked_mut(2) = Some(2);
    }

    assert_eq!(format!("{:?}", table), "{0: 1, 2: 2}");
  }
}
