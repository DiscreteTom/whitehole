use super::{SubTokenKind, TokenKindId, TokenKindIdProvider};

/// Bind the token kind value with an [`TokenKindId`].
/// This is readonly to make sure the binding is not broken.
/// # Examples
/// ```
/// use whitehole::lexer::token::{token_kind, TokenKindId, TokenKindIdBinding, TokenKindIdProvider, SubTokenKind};
///
/// #[token_kind]
/// #[derive(Debug)]
/// enum MyKind { A, B }
/// // struct `A` and `B` are generated by the macro
/// // and implement `Into<TokenKindIdBinding<MyKind>>`
///
/// # fn main() {
/// let a: TokenKindIdBinding<MyKind> = A.into();
/// let b: TokenKindIdBinding<MyKind> = B.into();
/// assert_eq!(a.id(), A::kind_id());
/// assert_eq!(b.id(), B::kind_id());
/// assert!(matches!(a.value(), MyKind::A));
/// assert!(matches!(b.value(), MyKind::B));
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TokenKindIdBinding<TokenKindType: 'static> {
  // this is `TokenKindId<TokenKindIdBinding<TokenKindType>>`
  // instead of `TokenKindId<TokenKindType>`
  // because the `kind_id` of generated structs are `TokenKindId<TokenKindIdBinding<TokenKindType>>`
  id: &'static TokenKindId<Self>,
  value: TokenKindType,
}

impl<TokenKindType> TokenKindIdProvider<Self> for TokenKindIdBinding<TokenKindType> {
  fn id(&self) -> &'static TokenKindId<Self> {
    &self.id
  }
}

// TODO: when rust support proxy pattern (not `Deref`), apply it here so that
// users can call methods with immutable ref on the value directly.
// e.g. `binding.method()` instead of `binding.value().method()`

impl<TokenKindType> TokenKindIdBinding<TokenKindType> {
  pub fn new<ViaKind: SubTokenKind<TokenKindIdBinding<TokenKindType>> + Into<TokenKindType>>(
    value: ViaKind,
  ) -> Self {
    Self {
      value: value.into(),
      id: ViaKind::kind_id(),
    }
  }

  pub fn value(&self) -> &TokenKindType {
    &self.value
  }

  /// Consume self and take the value out.
  pub fn take(self) -> TokenKindType {
    self.value
  }
}

/// Implement this trait for the token kind enum to provide the default token kind id binding.
/// This can be auto implemented by the [`token_kind`](crate::lexer::token::token_kind) macro.
/// # Examples
/// ```
/// use whitehole::lexer::token::{
///   token_kind, TokenKindId, TokenKindIdBinding, SubTokenKind, DefaultTokenKindIdBinding, TokenKindIdProvider,
/// };
///
/// #[token_kind]
/// #[derive(Default, Debug)]
/// enum MyKind { #[default] A }
///
/// # fn main() {
/// assert_eq!(MyKind::default_binding_kind_id(), A::kind_id());
/// assert!(matches!(MyKind::default(), MyKind::A));
///
/// // besides, `Default` will be implemented for `TokenKindIdBinding<MyKind>`
/// assert!(matches!(TokenKindIdBinding::<MyKind>::default().value(), MyKind::A));
/// assert_eq!(TokenKindIdBinding::<MyKind>::default().id(), A::kind_id());
/// # }
/// ```
pub trait DefaultTokenKindIdBinding<TokenKindType>: Default {
  fn default_binding_kind_id() -> &'static TokenKindId<TokenKindIdBinding<TokenKindType>>;
}

impl<TokenKindType: DefaultTokenKindIdBinding<TokenKindType>> Default
  for TokenKindIdBinding<TokenKindType>
{
  fn default() -> Self {
    Self {
      id: TokenKindType::default_binding_kind_id(),
      value: TokenKindType::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole_macros::_token_kind;

  #[_token_kind]
  #[derive(Debug, PartialEq, Default)]
  enum MyKind {
    #[default]
    A,
  }

  #[test]
  fn token_kind_id_binding() {
    let binding = TokenKindIdBinding::new(A);
    assert_eq!(binding.id(), &TokenKindId::new(0));
    assert_eq!(binding.value(), &MyKind::A);
    assert_eq!(binding.take(), MyKind::A);
  }

  #[test]
  fn default_token_kind_id_binding() {
    assert_eq!(MyKind::default_binding_kind_id(), A::kind_id());
    assert_eq!(MyKind::default(), MyKind::A);

    let binding = TokenKindIdBinding::<MyKind>::default();
    assert_eq!(binding.id(), &TokenKindId::new(0));
    assert_eq!(binding.value(), &MyKind::A);
  }
}
