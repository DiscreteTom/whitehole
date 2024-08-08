#[derive(Debug, Clone)]
pub(crate) struct LookupTable<V> {
  offset: usize,
  data: Vec<Option<V>>,
}

impl<V> LookupTable<V> {
  /// Return the value associated with the key.
  /// Return [`None`] if the key is not found or out of range.
  pub fn get(&self, key: usize) -> Option<&V> {
    if key < self.offset {
      None
    } else {
      self.data.get(key - self.offset).unwrap_or(&None).as_ref()
    }
  }
}

#[derive(Debug, Clone)]
pub(crate) struct CharLookupTableBuilder<V> {
  table: LookupTable<V>,
  /// Deduplicated keys, unordered.
  keys: Vec<char>,
}

macro_rules! get_unchecked_mut_by_char {
  ($self: expr, $c: expr) => {{
    let index = $c as usize - $self.table.offset;
    $self
      .table
      .data
      .get_unchecked_mut(index)
      .as_mut()
      .unwrap_unchecked()
  }};
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
      table: LookupTable { offset: min, data },
    }
  }

  /// Apply the function to each entry in the lookup table.
  pub fn for_each_entry_mut(&mut self, mut f: impl FnMut(char, &mut V)) {
    for k in &self.keys {
      // SAFETY: `k` is guaranteed to be in the range of `min..=max`.
      let d = unsafe { get_unchecked_mut_by_char!(self, *k) };
      f(*k, d);
    }
  }

  /// Apply the function to each value in the lookup table.
  pub fn for_each_value_mut(&mut self, mut f: impl FnMut(&mut V)) {
    for k in &self.keys {
      // SAFETY: `k` is guaranteed to be in the range of `min..=max`.
      let d = unsafe { get_unchecked_mut_by_char!(self, *k) };
      f(d);
    }
  }

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  pub unsafe fn get_unchecked_mut(&mut self, key: char) -> &mut V {
    get_unchecked_mut_by_char!(self, key)
  }

  /// Consume self, return [`LookupTable`].
  #[inline]
  pub fn build(self) -> LookupTable<V> {
    // discard keys since they are not used during runtime.
    // just return the lookup table.
    self.table
  }
}
