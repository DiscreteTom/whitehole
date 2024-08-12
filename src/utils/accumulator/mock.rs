use super::Accumulator;

impl<T> Accumulator<T> for () {
  #[inline]
  fn update(&mut self, _: T) {}
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mock_accumulator() {
    let mut acc = ();
    acc.update(123);
    assert_eq!(acc, ());
  }
}
