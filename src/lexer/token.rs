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

// make all fields public so the user can destruct the struct and get the fields
pub struct Token<'buffer, Kind, ErrorType> {
  /// The kind and the binding data.
  pub kind: Kind,
  /// The whole input text.
  pub buffer: &'buffer str,
  pub range: Range,
  pub error: Option<ErrorType>,
}

impl<'buffer, Kind, ErrorType> Token<'buffer, Kind, ErrorType> {
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
    let token = Token {
      kind: MyKind::UnitField,
      buffer,
      range: Range { start: 0, end: 3 },
      error: None::<()>,
    };
    assert!(matches!(token.kind, MyKind::UnitField));
    assert_eq!(token.buffer, buffer);
    assert_eq!(token.range.start, 0);
    assert_eq!(token.range.end, 3);
    assert_eq!(token.content(), "123");
    assert_eq!(token.error, None);
  }

  #[test]
  fn with_data() {
    let buffer = "123";
    let token = Token {
      kind: MyKind::UnnamedField(42),
      buffer,
      range: Range { start: 0, end: 3 },
      error: None::<()>,
    };
    assert!(matches!(token.kind, MyKind::UnnamedField(42)));
    assert_eq!(token.buffer, buffer);
    assert_eq!(token.range.start, 0);
    assert_eq!(token.range.end, 3);
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
