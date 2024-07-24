pub struct OneOrMore<T>(pub Vec<T>);

impl<T> From<T> for OneOrMore<T> {
  #[inline]
  fn from(value: T) -> Self {
    Self(vec![value])
  }
}
impl<T> From<Vec<T>> for OneOrMore<T> {
  #[inline]
  fn from(value: Vec<T>) -> Self {
    Self(value)
  }
}
impl<T, const N: usize> From<[T; N]> for OneOrMore<T> {
  #[inline]
  fn from(value: [T; N]) -> Self {
    Self(value.into())
  }
}

// additional implementations for OneOrMore<String>
impl From<&str> for OneOrMore<String> {
  #[inline]
  fn from(s: &str) -> Self {
    Self(vec![s.to_string()])
  }
}
impl From<Vec<&str>> for OneOrMore<String> {
  #[inline]
  fn from(ss: Vec<&str>) -> Self {
    Self(ss.into_iter().map(|s| s.to_string()).collect())
  }
}
impl<const N: usize> From<[&str; N]> for OneOrMore<String> {
  #[inline]
  fn from(ss: [&str; N]) -> Self {
    Self(ss.into_iter().map(|s| s.to_string()).collect())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn one_or_more_from_single() {
    let single: OneOrMore<i32> = OneOrMore::from(1);
    assert_eq!(single.0, vec![1]);
  }

  #[test]
  fn one_or_more_from_vec() {
    let vec: OneOrMore<i32> = OneOrMore::from(vec![1, 2, 3]);
    assert_eq!(vec.0, vec![1, 2, 3]);
  }

  #[test]
  fn one_or_more_from_array() {
    let array: OneOrMore<i32> = OneOrMore::from([1, 2, 3]);
    assert_eq!(array.0, vec![1, 2, 3]);
  }

  #[test]
  fn one_or_more_string_from_single() {
    let single: OneOrMore<String> = OneOrMore::from("a");
    assert_eq!(single.0, vec!["a".to_string()]);
  }

  #[test]
  fn one_or_more_string_from_vec() {
    let vec: OneOrMore<String> = OneOrMore::from(vec!["a", "b", "c"]);
    assert_eq!(
      vec.0,
      vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
  }

  #[test]
  fn one_or_more_string_from_array() {
    let array: OneOrMore<String> = OneOrMore::from(["a", "b", "c"]);
    assert_eq!(
      array.0,
      vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
  }
}
