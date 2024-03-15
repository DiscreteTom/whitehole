use super::TokenKindId;
use std::collections::HashSet;

pub trait TokenKind<TokenKindType> {
  // use associate type instead of generic type
  // because we want the token kind only have one possible target type
  /// For most cases this should be `Self`.
  /// ```
  /// # struct MyType(usize);
  /// impl TokenKind<MyType> for MyType {
  ///   type TargetType = Self;
  /// #   fn id(&self) -> &TokenKindId<Self> { &self.id }
  /// #   fn possible_kinds() -> HashSet<TokenKindId<Self::TargetType>> {
  /// #     HashSet::from([TokenKindId::new(0)])
  /// #   }
  /// }
  /// ```
  type TargetType; // TODO: defaults to TokenKindType. waiting https://github.com/rust-lang/rust/issues/29661

  /// The unique id of this token kind value.
  fn id(&self) -> &TokenKindId<TokenKindType>;
  /// Return a set containing all possible kind ids of this token kind.
  fn possible_kinds() -> HashSet<TokenKindId<Self::TargetType>>;
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
