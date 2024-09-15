use super::{offset::OffsetLookupTable, option::OptionLookupTable, Lookup};

/// Not every character is used in the lookup table, so we use [`OptionLookupTable`] to store values.
/// Since the range of characters is big and only a few characters are used, we use [`OffsetLookupTable`]
/// to wrap the [`OptionLookupTable`] to reduce memory usage.
pub type CharLookupTable<V> = OffsetLookupTable<OptionLookupTable<V>>;

/// In a lexer the range of known characters may be sparse.
/// E.g. in rust the smallest whitespace character is `0x0009` and the largest is `0x2029`.
/// If you use the [`CharLookupTable`] directly, it will allocate a lot of memory for unused characters.
///
/// However, known characters are usually clustered in smaller ranges.
/// E.g. in rust, whitespace characters can be divided into two ranges: `0x0009..=0x0085` and `0x200E..=0x2029`.
/// If we store each range in a separate [`CharLookupTable`], we can save a lot of memory
/// and speed up the building process.
///
/// In this struct we will split known characters into multiple [`CharLookupTable`]s.
/// Currently the algorithm is simple: sort all known characters and traverse them
/// (we don't even need to deduplicate them),
/// if the difference between two adjacent characters is greater than 128,
/// we split the before and after characters into two clusters.
///
/// Why 128? That's the range of ASCII characters, so we can ensure that
/// ASCII characters are stored in the same [`CharLookupTable`], and if the known characters only
/// contain ASCII characters, we only need one [`CharLookupTable`].
#[derive(Debug, Clone)]
pub struct SparseCharLookupTable<V> {
  /// The lookup tables for each cluster of characters.
  /// The clusters are ordered by its character range, from small to large,
  /// no overlap between clusters.
  ///
  /// E.g. `[0x0009..=0x0085, 0x200E..=0x2029]`.
  tables: Vec<CharLookupTable<V>>,
}

impl<V> Lookup for SparseCharLookupTable<V> {
  type Value = V;

  #[inline]
  fn get(&self, key: usize) -> Option<&Self::Value> {
    // TODO: do we need binary search here?
    self
      .tables
      .iter()
      .find(|table| key < table.len())
      .and_then(|table| table.get(key))
  }

  #[inline]
  fn len(&self) -> usize {
    self.tables.last().map_or(0, |table| table.len())
  }

  #[inline]
  unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut Self::Value {
    debug_assert!(key < self.len());

    // TODO: do we need binary search here?
    self
      .tables
      .iter_mut()
      .find(|table| key < table.len())
      .unwrap_unchecked()
      .get_unchecked_mut(key)
  }
}

#[derive(Debug, Clone)]
pub struct SparseCharLookupTableBuilder<V> {
  table: SparseCharLookupTable<V>,
  /// Deduplicated keys, ordered.
  /// One [`Vec`] for each cluster of characters.
  keys: Vec<Vec<char>>,
}

impl<V> SparseCharLookupTableBuilder<V> {
  /// # Caveats
  /// The caller must ensure that `raw_keys` is sorted and not empty.
  fn new_char_lookup_table(raw_keys: &[char]) -> (CharLookupTable<V>, Vec<char>)
  where
    V: Default,
  {
    debug_assert!(raw_keys.len() > 0);
    debug_assert!(raw_keys.windows(2).all(|w| w[0] <= w[1]));

    // SAFETY: `raw_keys` is not empty, so `min` is safe to be unchecked
    let min = *unsafe { raw_keys.get_unchecked(0) } as usize;
    let table =
      OptionLookupTable::with_keys_init(raw_keys.iter().map(|k| *k as usize - min), V::default);

    // here keys are ensured to be unique/deduplicated and ordered.
    let keys = table
      .keys()
      // SAFETY: these keys are safe to be transformed back to `char`
      .map(|k| unsafe { char::from_u32_unchecked((k + min) as u32) })
      .collect::<Vec<_>>();

    (CharLookupTable::new(min, table), keys)
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

    // there will be at least one table, so pre-allocate memory for it.
    let mut tables = Vec::with_capacity(1);
    let mut keys = Vec::with_capacity(1);

    // SAFETY: `raw_keys` is not empty, so `get(0)` is safe to be unchecked
    let mut last_traversed_char = *unsafe { raw_keys.get_unchecked(0) };
    let mut next_cluster_start_idx = 0;
    for (i, c) in raw_keys.iter().enumerate() {
      if (*c as usize) - (last_traversed_char as usize) > 128 {
        // SAFETY: `next_cluster_start_idx..i` is guaranteed to be in the range of `0..raw_keys.len()`.
        let slice = unsafe { raw_keys.get_unchecked(next_cluster_start_idx..i) };
        let (table, ks) = Self::new_char_lookup_table(slice);
        tables.push(table);
        keys.push(ks);
        next_cluster_start_idx = i;
      }
      // TODO: add a test to ensure the clustering is right
      last_traversed_char = *c;
    }
    // the last table
    // SAFETY: `next_cluster_start_idx..` is guaranteed to be in the range of `0..raw_keys.len()`.
    let slice = unsafe { raw_keys.get_unchecked(next_cluster_start_idx..) };
    let (table, ks) = Self::new_char_lookup_table(slice);
    tables.push(table);
    keys.push(ks);

    Self {
      keys,
      table: SparseCharLookupTable { tables },
    }
  }

  /// Apply the function to each entry in the lookup table.
  /// The traversal is ordered.
  pub fn for_each_entry_mut(&mut self, mut f: impl FnMut(char, &mut V)) {
    // since we stored keys, accessing using keys should be faster than traversing the underlying OptionLookupTable
    for (keys, table) in self.keys.iter().zip(self.table.tables.iter_mut()) {
      for k in keys {
        // SAFETY: `k` is guaranteed to be a key of `table`
        let d = unsafe { table.get_unchecked_mut(*k as usize) };
        f(*k, d);
      }
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
