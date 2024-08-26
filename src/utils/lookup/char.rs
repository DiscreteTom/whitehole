use super::{lookup::Lookup, offset::OffsetLookupTable, option::OptionLookupTable};
use std::hint::unreachable_unchecked;

/// Not every character is used in the lookup table, so we use [`OptionLookupTable`] to store values.
/// Since the range of characters is big and only a few characters are used, we use [`OffsetLookupTable`]
/// to wrap the [`OptionLookupTable`] to reduce memory usage.
pub(crate) type CharLookupTable<V> = OffsetLookupTable<OptionLookupTable<V>>;

#[derive(Debug, Clone)]
pub(crate) struct SparseCharLookupTable<V> {
  tables: Vec<CharLookupTable<V>>,
}

impl<V> Lookup for SparseCharLookupTable<V> {
  type Value = V;

  #[inline]
  fn get(&self, key: usize) -> Option<&Self::Value> {
    for table in &self.tables {
      if key < table.len() {
        return table.get(key);
      }
    }
    None
  }

  #[inline]
  fn len(&self) -> usize {
    self.tables.last().map_or(0, |table| table.len())
  }

  #[inline]
  unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut Self::Value {
    for table in &mut self.tables {
      if key < table.len() {
        return table.get_unchecked_mut(key);
      }
    }
    unreachable_unchecked()
  }
}

#[derive(Debug, Clone)]
pub(crate) struct SparseCharLookupTableBuilder<V> {
  table: SparseCharLookupTable<V>,
  /// Deduplicated keys, ordered.
  keys: Vec<char>,
}

impl<V> SparseCharLookupTableBuilder<V> {
  /// Caveats
  /// The caller must ensure that `raw_keys` is sorted and not empty.
  fn new_char_lookup_table(raw_keys: &[char], keys: &mut Vec<char>) -> CharLookupTable<V>
  where
    V: Default,
  {
    // SAFETY: `raw_keys` is not empty, so `min` and `max` are safe to be unchecked
    let min = *unsafe { raw_keys.get_unchecked(0) } as usize;
    let max = *unsafe { raw_keys.get_unchecked(raw_keys.len() - 1) } as usize;
    let size = max - min + 1;
    let mut table = OptionLookupTable::new(size);

    for k in raw_keys {
      // SAFETY: `k` is guaranteed to be in the range of `min..=max`.
      let d = unsafe { table.get_option_unchecked_mut(*k as usize - min) };
      if d.is_none() {
        *d = Some(V::default());
        keys.push(*k); // keys are ensured to be unique/deduplicated.
      }
    }

    CharLookupTable::new(min, table)
  }

  /// Create a new instance with the given keys.
  /// Keys can be empty, unordered or duplicated.
  pub fn new(mut raw_keys: Vec<char>) -> Self
  where
    V: Default,
  {
    if raw_keys.is_empty() {
      return Self {
        keys: Vec::new(),
        table: SparseCharLookupTable { tables: Vec::new() },
      };
    }

    raw_keys.sort();

    // there will be at least one table
    let mut tables = Vec::with_capacity(1);
    // pre-allocate memory for keys with the same size as `raw_keys`. (assume no duplicated keys)
    // `keys.len()` will be less than or equal to `raw_keys.len()`.
    // don't use `size` as the capacity because it may be much larger than `raw_keys.len()`
    // if the keys are sparse.
    let mut keys = Vec::with_capacity(raw_keys.len());

    // SAFETY: `raw_keys` is not empty, so `last` is safe to be unchecked
    let mut last = *unsafe { raw_keys.get_unchecked(0) };
    let mut start = 0;
    for (i, c) in raw_keys.iter().enumerate() {
      if (*c as usize) - (last as usize) > 128 {
        let slice = unsafe { raw_keys.get_unchecked(start..i) };
        tables.push(Self::new_char_lookup_table(slice, &mut keys));

        last = *c;
        start = i;
      }
    }
    // the last table
    let slice = unsafe { raw_keys.get_unchecked(start..) };
    tables.push(Self::new_char_lookup_table(slice, &mut keys));

    Self {
      keys,
      table: SparseCharLookupTable { tables },
    }
  }

  /// Apply the function to each entry in the lookup table.
  /// The traversal is unordered.
  pub fn for_each_entry_mut(&mut self, mut f: impl FnMut(char, &mut V)) {
    for k in &self.keys {
      // SAFETY: `k` is guaranteed to be a key of `self.table`
      let d = unsafe { self.table.get_unchecked_mut(*k as usize) };
      f(*k, d);
    }
  }

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  /// # Panics
  /// Panics if the key is smaller than the minimum key provided in [`Self::new`].
  pub unsafe fn get_unchecked_mut(&mut self, key: char) -> &mut V {
    self.table.get_unchecked_mut(key as usize)
  }

  /// Consume self, return [`SparseCharLookupTable`].
  #[inline]
  pub fn build(self) -> SparseCharLookupTable<V> {
    // discard keys since they are not used during runtime.
    // just return the lookup table.
    self.table
  }
}

// TODO: add tests
// #[cfg(test)]
// mod tests {
//   use super::*;

//   #[test]
//   fn test_empty_char_lookup_table_builder() {
//     let builder = SparseCharLookupTableBuilder::<u8>::new(&[]);
//     assert_eq!(builder.table.get(0), None);
//   }

//   #[test]
//   fn test_char_lookup_table_builder() {
//     // unordered, duplicated keys
//     let keys = ['a', 'b', 'a', 'c'];
//     let mut builder = SparseCharLookupTableBuilder::new(&keys);

//     builder.for_each_entry_mut(|k, v| *v = k);

//     assert_eq!(builder.table.get('a' as usize), Some(&'a'));
//     assert_eq!(builder.table.get('b' as usize), Some(&'b'));
//     assert_eq!(builder.table.get('c' as usize), Some(&'c'));

//     unsafe {
//       *builder.get_unchecked_mut('a') = 'd';
//       *builder.get_unchecked_mut('b') = 'e';
//       *builder.get_unchecked_mut('c') = 'f';
//     }

//     assert_eq!(builder.table.get('a' as usize), Some(&'d'));
//     assert_eq!(builder.table.get('b' as usize), Some(&'e'));
//     assert_eq!(builder.table.get('c' as usize), Some(&'f'));
//   }
// }
