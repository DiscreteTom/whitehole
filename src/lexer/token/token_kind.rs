use super::TokenKindId;
use std::collections::HashSet;

/// If a type implement this, all the possible [`TokenKindId`]
/// should be able to be retrieved by [`TokenKind::possible_kinds`].
/// This can be auto implemented by deriving [`whitehole_macros::TokenKind`].
/// # Examples
/// ```
/// use std::collections::HashSet;
/// use whitehole_macros::TokenKind;
/// use whitehole::lexer::token::{TokenKindId, TokenKind};
///
/// #[derive(TokenKind)]
/// enum MyKind { A, B }
///
/// assert_eq!(MyKind::possible_kinds(), HashSet::from([
///   TokenKindId::new(0),
///   TokenKindId::new(1)
/// ]));
/// ```
pub trait TokenKind<TokenKindType> {
  /// Return a set containing all possible kind ids of this token kind.
  fn possible_kinds() -> HashSet<TokenKindId<TokenKindType>>;
}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole_macros::_TokenKind;
  use MyKind::*;

  #[derive(_TokenKind)]
  enum MyKind {
    UnitField,
    UnnamedField(i32),
    NamedField { _a: i32 },
  }

  #[test]
  fn token_kind_id() {
    assert_eq!(UnitField.id().0, 0);
    assert_eq!(UnnamedField(42).id().0, 1);
    assert_eq!(NamedField { _a: 1 }.id().0, 2);
  }
}
