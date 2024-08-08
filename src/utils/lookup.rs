#[derive(Clone)]
pub struct CharLookupBuilder<V> {
  min: usize,
  keys: Vec<char>,
  data: Vec<Option<V>>,
}

impl<V> CharLookupBuilder<V> {
  /// Create a new lookup table with the given keys.
  /// Keys can be empty, unordered or duplicated.
  pub fn new(raw_keys: &[char]) -> Self
  where
    V: Default,
  {
    let min = *raw_keys.iter().min().unwrap_or(&'\0') as usize;
    let max = *raw_keys.iter().max().unwrap_or(&'\0') as usize;
    let mut keys = Vec::with_capacity(raw_keys.len());

    let mut data: Vec<Option<V>> = Vec::with_capacity(max - min + 1);
    data.resize_with(max - min + 1, || None);

    for k in raw_keys {
      // SAFETY: `k` is guaranteed to be in the range of `min..=max`.
      let d = unsafe { data.get_unchecked_mut(*k as usize - min) };
      if d.is_none() {
        *d = Some(V::default());
        keys.push(*k);
      }
    }

    Self { min, data, keys }
  }

  /// Return the value associated with the key.
  /// Return [`None`] if the key is not found or out of range.
  pub fn get(&self, key: char) -> &Option<V> {
    if (key as usize) < self.min {
      return &None;
    }
    self.data.get(key as usize - self.min).unwrap_or(&None)
  }

  pub fn for_each_mut(&mut self, mut f: impl FnMut(char, &mut V)) {
    for k in &self.keys {
      let d = unsafe {
        self
          .data
          .get_unchecked_mut(*k as usize - self.min)
          .as_mut()
          .unwrap_unchecked()
      };
      f(*k, d);
    }
  }

  pub fn for_each_value_mut(&mut self, mut f: impl FnMut(&mut V)) {
    for k in &self.keys {
      let d = unsafe {
        self
          .data
          .get_unchecked_mut(*k as usize - self.min)
          .as_mut()
          .unwrap_unchecked()
      };
      f(d);
    }
  }

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  pub unsafe fn get_unchecked_mut(&mut self, key: char) -> &mut V {
    self
      .data
      .get_unchecked_mut(key as usize - self.min)
      .as_mut()
      .unwrap_unchecked()
  }
}
