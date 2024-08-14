use super::{lookup::Lookup, offset::OffsetLookupTable, option::OptionLookupTable};

/// Not every character is used in the lookup table, so we use [`OptionLookupTable`] to store values.
/// Since the range of characters is big and only a few characters are used, we use [`OffsetLookupTable`]
/// to wrap the [`OptionLookupTable`] to reduce memory usage.
pub(crate) type CharLookupTable<V> = OffsetLookupTable<OptionLookupTable<V>>;

#[derive(Debug, Clone)]
pub(crate) struct CharLookupTableBuilder<V> {
  table: CharLookupTable<V>,
  /// Deduplicated keys, unordered.
  keys: Vec<char>,
}

impl<V> CharLookupTableBuilder<V> {
  /// Create a new instance with the given keys.
  /// Keys can be empty, unordered or duplicated.
  /// # Design
  /// We use a char slice as the parameter, so the caller
  /// doesn't need to use [`HashMap`](std::collections::HashMap) to deduplicate keys
  /// (checking duplication in a lookup table is more efficient).
  pub fn new(raw_keys: &[char]) -> Self
  where
    V: Default,
  {
    if raw_keys.is_empty() {
      return Self {
        keys: Vec::new(),
        table: CharLookupTable::new(0, OptionLookupTable::new(0)),
      };
    }

    // SAFETY: `raw_keys` is not empty, so `min` and `max` are safe to unwrap.
    let min = *unsafe { raw_keys.iter().min().unwrap_unchecked() } as usize;
    let max = *unsafe { raw_keys.iter().max().unwrap_unchecked() } as usize;
    let size = max - min + 1;
    let mut table = OptionLookupTable::new(size);

    // pre-allocate memory for keys with the same size as `raw_keys`. (assume no duplicated keys)
    // `keys.len()` will be less than or equal to `raw_keys.len()`.
    // don't use `size` as the capacity because it may be much larger than `raw_keys.len()`
    // if the keys are sparse.
    let mut keys = Vec::with_capacity(raw_keys.len());

    for k in raw_keys {
      // SAFETY: `k` is guaranteed to be in the range of `min..=max`.
      let d = unsafe { table.get_option_unchecked_mut(*k as usize - min) };
      if d.is_none() {
        *d = Some(V::default());
        keys.push(*k); // keys are ensured to be unique/deduplicated.
      }
    }

    Self {
      keys,
      table: CharLookupTable::new(min, table),
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

  /// Consume self, return [`CharLookupTable`].
  #[inline]
  pub fn build(self) -> CharLookupTable<V> {
    // discard keys since they are not used during runtime.
    // just return the lookup table.
    self.table
  }
}
