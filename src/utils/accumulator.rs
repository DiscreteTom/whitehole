/// Accumulate values.
///
/// These types already implement the [`Accumulator`] trait:
/// - `()` - mock accumulator that does nothing.
/// - [`Vec<T>`] - accumulate values into a vector.
/// - [`String`] - accumulate characters or strings into a string.
pub trait Accumulator<T> {
  /// Update the accumulator with a value.
  fn update(&mut self, t: T);
}

// mock accumulator
impl<T> Accumulator<T> for () {
  #[inline]
  fn update(&mut self, _: T) {}
}

// vector accumulator
impl<T> Accumulator<T> for Vec<T> {
  #[inline]
  fn update(&mut self, c: T) {
    self.push(c);
  }
}

// string accumulator
impl Accumulator<char> for String {
  #[inline]
  fn update(&mut self, c: char) {
    self.push(c);
  }
}
impl Accumulator<String> for String {
  #[inline]
  fn update(&mut self, c: String) {
    self.push_str(&c);
  }
}
impl Accumulator<&str> for String {
  #[inline]
  fn update(&mut self, c: &str) {
    self.push_str(c);
  }
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

  #[test]
  fn vec_accumulator() {
    let mut acc = vec![];
    acc.update(1);
    acc.update(2);
    acc.update(3);
    assert_eq!(acc, vec![1, 2, 3]);
  }

  #[test]
  fn string_accumulator() {
    let mut acc = String::new();
    acc.update('1');
    acc.update('2');
    acc.update('3');
    acc.update("456".to_string());
    assert_eq!(acc, "123456");
  }
}
