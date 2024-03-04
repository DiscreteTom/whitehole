pub struct StringList(pub Vec<String>);

impl From<String> for StringList {
  fn from(s: String) -> Self {
    StringList(vec![s])
  }
}

impl From<&str> for StringList {
  fn from(s: &str) -> Self {
    StringList(vec![s.to_string()])
  }
}

impl From<Vec<String>> for StringList {
  fn from(ss: Vec<String>) -> Self {
    StringList(ss)
  }
}

impl<const N: usize> From<[String; N]> for StringList {
  fn from(ss: [String; N]) -> Self {
    StringList(ss.to_vec())
  }
}
impl<const N: usize> From<[&str; N]> for StringList {
  fn from(ss: [&str; N]) -> Self {
    StringList(ss.iter().map(|s| s.to_string()).collect())
  }
}
