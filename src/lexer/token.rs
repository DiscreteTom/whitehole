pub mod buffer;

use self::buffer::CowString;

/// The unique identifier of a token kind.
/// Usually we use enum variants as token kinds, and the identifier is the variant's index.
pub type TokenKindId = usize;

pub trait TokenKind {
  fn id(&self) -> TokenKindId;
}

pub struct Token<Kind, ErrorType> {
  // TODO: make fields private
  /// The kind and the binding data.
  pub kind: Kind,
  /// The whole input text.
  pub buffer: CowString,
  /// The index of the first character of the token in the whole input text.
  pub start: usize,
  /// The index of the last character of the token in the whole input text.
  pub end: usize,
  pub error: Option<ErrorType>,
}

impl<Kind, ErrorType> Token<Kind, ErrorType> {
  /// Returns the content of the token.
  pub fn content(&self) -> &str {
    &self.buffer.value()[self.start..self.end]
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
    let buffer = CowString::new("123");
    let token = Token {
      kind: MyKind::UnitField,
      buffer: buffer.clone(),
      start: 0,
      end: 3,
      error: None::<()>,
    };
    assert!(matches!(token.kind, MyKind::UnitField));
    assert_eq!(token.buffer.value(), buffer.value());
    assert_eq!(token.start, 0);
    assert_eq!(token.end, 3);
    assert_eq!(token.content(), "123");
    assert_eq!(token.error, None);
  }

  #[test]
  fn with_data() {
    let buffer = CowString::new("123");
    let token = Token {
      kind: MyKind::UnnamedField(42),
      buffer: buffer.clone(),
      start: 0,
      end: 3,
      error: None::<()>,
    };
    assert!(matches!(token.kind, MyKind::UnnamedField(42)));
    assert_eq!(token.buffer.value(), buffer.value());
    assert_eq!(token.start, 0);
    assert_eq!(token.end, 3);
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
