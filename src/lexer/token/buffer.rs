use std::rc::Rc;

// TODO: maybe use std::Cow?
/// Copy-on-write string.
#[derive(Clone)]
pub struct CowString {
  value: Rc<String>,
}

// DON'T implement `Deref` for `CowString` for simplicity
// since the inner string methods may change the string value

impl Default for CowString {
  fn default() -> Self {
    CowString {
      value: Rc::new(String::new()),
    }
  }
}

impl CowString {
  pub fn new(s: &str) -> Self {
    CowString {
      value: Rc::new(s.into()),
    }
  }

  pub fn value(&self) -> &str {
    self.value.as_str()
  }

  pub fn reset(&mut self) {
    self.value = Rc::new(String::new());
  }

  pub fn feed(&mut self, s: &str) {
    let buffer = self.value.clone();
    self.value = Rc::new((*buffer).clone() + s);
  }
}
