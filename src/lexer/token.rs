/// The unique identifier of a token kind.
/// Usually we use enum variants as token kinds, and the identifier is the variant's index.
pub type TokenKindId = usize;

pub trait TokenKind {
  fn id(&self) -> TokenKindId;
}

pub struct Token<'buffer, Kind, ErrorType> {
  /// The kind and the binding data.
  kind: Kind,
  /// The whole input text.
  buffer: &'buffer str,
  /// The index of the first character of the token in the whole input text.
  start: usize,
  /// The index of the last character of the token in the whole input text.
  end: usize,
  error: Option<ErrorType>,
}

impl<'buffer, Kind, ErrorType> Token<'buffer, Kind, ErrorType> {
  /// Returns the kind (and the binding data) of the token.
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

  pub fn error(&self) -> Option<&ErrorType> {
    self.error.as_ref()
  }

  /// Returns the content of the token.
  pub fn content(&self) -> &str {
    &self.buffer[self.start..self.end]
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole_macros::TokenKind;

  #[derive(TokenKind)]
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
      error: None::<()>,
    };
    assert!(matches!(token.kind(), MyKind::Simple));
    assert_eq!(token.buffer(), buffer);
    assert_eq!(token.start(), 0);
    assert_eq!(token.end(), 3);
    assert_eq!(token.content(), "123");
    assert_eq!(token.error(), None);
  }

  #[test]
  fn with_data() {
    let buffer = "123";
    let token = Token {
      kind: MyKind::WithData(42),
      buffer,
      start: 0,
      end: 3,
      error: None::<()>,
    };
    assert!(matches!(token.kind(), MyKind::WithData(42)));
    assert_eq!(token.buffer(), buffer);
    assert_eq!(token.start(), 0);
    assert_eq!(token.end(), 3);
    assert_eq!(token.content(), "123");
    assert_eq!(token.error(), None);
  }

  #[test]
  fn token_kind_id() {
    assert_eq!(MyKind::Simple.id(), 0);
    assert_eq!(MyKind::WithData(42).id(), 1);
  }
}
