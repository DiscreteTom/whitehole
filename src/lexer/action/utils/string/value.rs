pub trait PartialStringBodyValue: Default {
  fn from_str(s: &str) -> Self;
  fn from_char(c: char) -> Self;
}

impl PartialStringBodyValue for String {
  fn from_str(s: &str) -> Self {
    s.to_string()
  }
  fn from_char(c: char) -> Self {
    c.to_string()
  }
}

impl PartialStringBodyValue for () {
  fn from_str(_: &str) -> Self {}
  fn from_char(_: char) -> Self {}
}
