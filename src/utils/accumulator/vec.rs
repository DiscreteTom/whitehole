use super::Accumulator;

impl<T> Accumulator<T> for Vec<T> {
  #[inline]
  fn update(&mut self, c: T) {
    self.push(c);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn vec_accumulator() {
    let mut acc = vec![];
    acc.update(1);
    acc.update(2);
    acc.update(3);
    assert_eq!(acc, vec![1, 2, 3]);
  }
}
