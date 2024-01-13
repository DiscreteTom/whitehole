pub struct Token<'buffer, Kind> {
  /// The kind and the binding data.
  kind: Kind,
  /// The whole input text.
  buffer: &'buffer str,
  /// The index of the first character of the token in the whole input text.
  start: usize,
  /// The index of the last character of the token in the whole input text.
  end: usize,
  // TODO: add error
}

impl<'buffer, Kind> Token<'buffer, Kind> {
  /// Returns the kind of the token.
  pub fn kind(&self) -> &Kind {
    &self.kind
  }

  /// Returns the whole input text.
  pub fn buffer(&self) -> &'buffer str {
    self.buffer
  }

  /// Returns the index of the first character of the token in the whole input text.
  pub fn start(&self) -> usize {
    self.start
  }

  /// Returns the index of the last character of the token in the whole input text.
  pub fn end(&self) -> usize {
    self.end
  }

  /// Returns the content of the token.
  pub fn content(&self) -> &str {
    &self.buffer[self.start..self.end]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  enum MyKind {
    Simple,
    WithData(i32),
  }

  #[test]
  fn simple() {
    let buffer = "123";
    let token = Token {
      kind: MyKind::Simple,
      buffer,
      start: 0,
      end: 3,
    };
    assert!(matches!(token.kind(), MyKind::Simple));
    assert_eq!(token.buffer(), buffer);
    assert_eq!(token.start(), 0);
    assert_eq!(token.end(), 3);
    assert_eq!(token.content(), "123");
  }

  #[test]
  fn with_data() {
    let buffer = "123";
    let token = Token {
      kind: MyKind::WithData(42),
      buffer,
      start: 0,
      end: 3,
    };
    assert!(matches!(token.kind(), MyKind::WithData(42)));
    assert_eq!(token.buffer(), buffer);
    assert_eq!(token.start(), 0);
    assert_eq!(token.end(), 3);
    assert_eq!(token.content(), "123");
  }
}
