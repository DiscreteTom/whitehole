use super::Accumulator;

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
  fn string_accumulator() {
    let mut acc = String::new();
    acc.update('1');
    acc.update('2');
    acc.update('3');
    acc.update("456".to_string());
    acc.update("789");
    assert_eq!(acc, "123456789");
  }
}
