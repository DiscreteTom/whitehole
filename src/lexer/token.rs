/// The unique identifier of a token kind.
/// Usually we use enum variants as token kinds, and the identifier is the variant's index.
pub type TokenKindId = usize;

pub trait TokenKind {
  fn id(&self) -> TokenKindId;
}

#[derive(Debug)]
pub struct Range {
  /// 0-based index.
  pub start: usize,
  /// 0-based index. Exclusive.
  pub end: usize,
}

pub struct Token<'buffer, Kind, ErrorType> {
  /// The kind and the binding data.
  kind: Kind,
  /// The whole input text.
  buffer: &'buffer str,
  range: Range,
  error: Option<ErrorType>,
}

impl<'buffer, Kind, ErrorType> Token<'buffer, Kind, ErrorType> {
  pub fn new(
    kind: Kind,
    buffer: &'buffer str,
    start: usize,
    end: usize,
    error: Option<ErrorType>,
  ) -> Self {
    Token {
      kind,
      buffer,
      range: Range { start, end },
      error,
    }
  }

  pub fn kind(&self) -> &Kind {
    &self.kind
  }
  pub fn buffer(&self) -> &'buffer str {
    self.buffer
  }
  pub fn range(&self) -> &Range {
    &self.range
  }
  pub fn start(&self) -> usize {
    self.range.start
  }
  pub fn end(&self) -> usize {
    self.range.end
  }
  pub fn error(&self) -> &Option<ErrorType> {
    &self.error
  }

  /// Returns the content of the token.
  pub fn content(&self) -> &str {
    &self.buffer[self.range.start..self.range.end]
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole_macros::TokenKind;

  #[derive(TokenKind)]
  enum MyKind {
    UnitField,
    UnnamedField(i32),
    NamedField { _a: i32 },
  }

  #[test]
  fn simple() {
    let buffer = "123";
    let token = Token::new(MyKind::UnitField, buffer, 0, 3, None::<()>);
    assert!(matches!(token.kind, MyKind::UnitField));
    assert_eq!(token.buffer, buffer);
    assert_eq!(token.start(), 0);
    assert_eq!(token.end(), 3);
    assert_eq!(token.content(), "123");
    assert_eq!(token.error, None);
  }

  #[test]
  fn with_data() {
    let buffer = "123";
    let token = Token::new(MyKind::UnnamedField(42), buffer, 0, 3, None::<()>);
    assert!(matches!(token.kind, MyKind::UnnamedField(42)));
    assert_eq!(token.buffer, buffer);
    assert_eq!(token.start(), 0);
    assert_eq!(token.end(), 3);
    assert_eq!(token.content(), "123");
    assert_eq!(token.error, None);
  }

  #[test]
  fn token_kind_id() {
    assert_eq!(MyKind::UnitField.id(), 0);
    assert_eq!(MyKind::UnnamedField(42).id(), 1);
    assert_eq!(MyKind::NamedField { _a: 1 }.id(), 2);
  }
}
