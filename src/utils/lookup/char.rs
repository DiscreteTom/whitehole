use super::{
  offset::OffsetLookupTable,
  option::{Lookup, OptionLookupTable},
};

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
  // TODO: better parameter type?
  pub fn new(raw_keys: &[char]) -> Self
  where
    V: Default,
  {
    let min = *raw_keys.iter().min().unwrap_or(&'\0') as usize;
    let max = *raw_keys.iter().max().unwrap_or(&'\0') as usize;
    let size = max - min + 1;
    let mut table = OptionLookupTable::new(size);

    // pre-allocate memory for keys, assume no duplicated keys.
    let mut keys = Vec::with_capacity(raw_keys.len());

    for k in raw_keys {
      // SAFETY: `k` is guaranteed to be in the range of `min..=max`.
      let d = unsafe { table.get_option_unchecked_mut(*k as usize - min) };
      if d.is_none() {
        *d = Some(V::default());
        keys.push(*k);
      }
    }

    Self {
      keys,
      table: CharLookupTable::new(min, table),
    }
  }

  /// Apply the function to each entry in the lookup table.
  pub fn for_each_entry_mut(&mut self, mut f: impl FnMut(char, &mut V)) {
    for k in &self.keys {
      // SAFETY: `k` is guaranteed to be in the range of `min..=max`.
      let d = unsafe { self.table.get_unchecked_mut(*k as usize) };
      f(*k, d);
    }
  }

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  /// # Panics
  /// Panics if the key is smaller than the offset.
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
