#[derive(Debug, Clone)]
pub(crate) struct LookupTable<V> {
  data: Vec<Option<V>>,
}

impl<V> LookupTable<V> {
  /// Return the value associated with the key.
  /// Return [`None`] if the key is not found or out of range.
  pub fn get(&self, key: usize) -> Option<&V> {
    self.data.get(key).unwrap_or(&None).as_ref()
  }

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut V {
    self.data.get_unchecked_mut(key).as_mut().unwrap_unchecked()
  }
}

/// A lookup table that the first `n` keys are empty.
/// So the offset lookup table will record the offset
/// to prevent allocating unnecessary memory.
#[derive(Debug, Clone)]
pub(crate) struct OffsetLookupTable<V> {
  offset: usize,
  table: LookupTable<V>,
}

impl<V> OffsetLookupTable<V> {
  /// Return the value associated with the key.
  /// Return [`None`] if the key is not found or out of range.
  pub fn get(&self, key: usize) -> Option<&V> {
    if key < self.offset {
      None
    } else {
      self.table.get(key - self.offset)
    }
  }

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  /// # Panics
  /// Panics if the key is smaller than the offset.
  #[inline]
  unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut V {
    self.table.get_unchecked_mut(key - self.offset)
  }
}

#[derive(Debug, Clone)]
pub(crate) struct CharLookupTableBuilder<V> {
  table: OffsetLookupTable<V>,
  /// Deduplicated keys, unordered.
  keys: Vec<char>,
}

impl<V> CharLookupTableBuilder<V> {
  /// Create a new lookup table with the given keys.
  /// Keys can be empty, unordered or duplicated.
  // TODO: better parameter type?
  pub fn new(raw_keys: &[char]) -> Self
  where
    V: Default,
  {
    let min = *raw_keys.iter().min().unwrap_or(&'\0') as usize;
    let max = *raw_keys.iter().max().unwrap_or(&'\0') as usize;
    let size = max - min + 1;
    let mut keys = Vec::with_capacity(raw_keys.len());

    let mut data: Vec<Option<V>> = Vec::with_capacity(size);
    data.resize_with(size, || None);

    for k in raw_keys {
      // SAFETY: `k` is guaranteed to be in the range of `min..=max`.
      let d = unsafe { data.get_unchecked_mut(*k as usize - min) };
      if d.is_none() {
        *d = Some(V::default());
        keys.push(*k);
      }
    }

    Self {
      keys,
      table: OffsetLookupTable {
        offset: min,
        table: LookupTable { data },
      },
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

  /// Consume self, return [`LookupTable`].
  #[inline]
  pub fn build(self) -> OffsetLookupTable<V> {
    // discard keys since they are not used during runtime.
    // just return the lookup table.
    self.table
  }
}
