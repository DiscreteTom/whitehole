//! ## Design
//!
//! Usually when we want to write a lexer, we need to define "token kinds",
//! like `Identifier`, `Number`, etc.
//! We can use enum to represent these kinds.
//!
//! ```
//! pub enum MyKind {
//!   Identifier,
//!   Number,
//! }
//! ```
//!
//! Besides, we may want to carry some data with different token kinds.
//! The data may be generated during the lexing process and stored in the token
//! so we don't need to parse the token content again after lexing.
//! An example is that if we want to lex a string literal with escape sequences,
//! when the token is yielded we should already know the evaluated value of the string literal,
//! we can store the value in the token, instead of parsing the literal content again.
//! The data should be associated with the token kind,
//! so we can use enum variants to represent them.
//!
//! ```
//! pub enum MyKind {
//!   Identifier(String),
//!   Number(i32),
//! }
//! ```
//!
//! However, in rust we treat `Number(0)` and `Number(1)` as different values,
//! but their token kinds are the same.
//! To solve this problem, we can use a [`TokenKindId`] to identify different token kinds.
//! `Number(0)` and `Number(1)` are different values but they have the same [`TokenKindId`].
//! We use the index of the enum variant as the id of the token kind, so in the example above,
//! `Identifier` has id `0` and `Number` has id `1`.
//! By doing this, `Number(0)` and `Number(1)` have the same token kind id `1`.
//! (Why not just use [`std::mem::Discriminant`]? See [`TokenKindId`] for more details.)
//!
//! We also need a way to get the token kind id from a token kind value.
//! An easy way is to use pattern matching like this:
//!
//! ```
//! # pub enum MyKind {
//! #   Identifier(String),
//! #   Number(i32),
//! # }
//! fn get_id(kind: MyKind) -> usize {
//!   match kind {
//!     MyKind::Identifier(_) => 0,
//!     MyKind::Number(_) => 1,
//!   }
//! }
//! ```
//!
//! However we will access the token kind id frequently, so we store the id and the token kind value together
//! to prevent unnecessary pattern matching (just like cache the result of the pattern matching).
//! We use [`TokenKindIdBinding`] to bind the id and the value.
//!
//! ```
//! # use whitehole::lexer::token::TokenKindId;
//! # pub enum MyKind {
//! #   Identifier(String),
//! #   Number(i32),
//! # }
//! #
//! pub struct TokenKindIdBinding<TokenKindType> {
//!   id: TokenKindId<TokenKindIdBinding<TokenKindType>>,
//!   value: TokenKindType,
//! };
//!
//! // when creating `TokenKindIdBinding`, we have to make sure
//! // the id and the value are bound correctly.
//!
//! // correct
//! TokenKindIdBinding {
//!   id: TokenKindId::new(0), // the id of `Identifier`
//!   value: MyKind::Identifier("hello".to_string())
//! };
//! TokenKindIdBinding {
//!   id: TokenKindId::new(1), // the id of `Number`
//!   value: MyKind::Number(0)
//! };
//!
//! // wrong!
//! TokenKindIdBinding {
//!   id: TokenKindId::new(0), // the id of `Identifier`
//!   value: MyKind::Number(0)
//! };
//! ```
//!
//! As you can see, we want to get the id bound with a value of `MyKind`, and we get the id from [`TokenKindIdBinding`].
//! Thus the [`TokenKindIdBinding`] is a [`TokenKindIdProvider`], `MyKind` is not.
//!
//! To achieve the strict binding between the token kind id and the token kind value,
//! we will create structs for each enum variant and implement `Into<TokenKindIdBinding<MyKind>>` for them.
//!
//! ```
//! # use whitehole::lexer::token::{TokenKindId};
//! # pub struct TokenKindIdBinding<TokenKindType> {
//! #   id: TokenKindId<TokenKindIdBinding<TokenKindType>>,
//! #   value: TokenKindType,
//! # };
//! // this is the "token kind"
//! pub enum MyKind {
//!   // instead of storing the token value directly,
//!   // we store "sub token kind" values in the enum variant
//!   // to avoid destructing sub token kind values
//!   // when constructing token kind values
//!   Identifier(Identifier),
//!   Number(Number),
//! }
//!
//! // these are "sub token kind"s, they store token's data
//! pub struct Identifier(pub String);
//! pub struct Number(pub i32);
//!
//! // sub token kinds can be converted into the token kind
//! impl Into<MyKind> for Identifier {
//!   fn into(self) -> MyKind {
//!     MyKind::Identifier(self)
//!   }
//! }
//! impl Into<MyKind> for Number {
//!   fn into(self) -> MyKind {
//!     MyKind::Number(self)
//!   }
//! }
//!
//! pub trait SubTokenKind<Kind> {
//!   fn kind_id() -> TokenKindId<Kind>;
//! }
//!
//! // every sub token kind should have a unique id
//! // bound with the type, not its value
//! impl SubTokenKind<TokenKindIdBinding<MyKind>> for Identifier {
//!   fn kind_id() -> TokenKindId<TokenKindIdBinding<MyKind>> {
//!     TokenKindId::new(0)
//!   }
//! }
//! impl SubTokenKind<TokenKindIdBinding<MyKind>> for Number {
//!   fn kind_id() -> TokenKindId<TokenKindIdBinding<MyKind>> {
//!     TokenKindId::new(1)
//!   }
//! }
//!
//! // from sub token kinds we can create the token kind id bindings
//! impl Into<TokenKindIdBinding<MyKind>> for Identifier {
//!   fn into(self) -> TokenKindIdBinding<MyKind> {
//!     TokenKindIdBinding { id: Identifier::kind_id(), value: MyKind::Identifier(self) }
//!   }
//! }
//! impl Into<TokenKindIdBinding<MyKind>> for Number {
//!   fn into(self) -> TokenKindIdBinding<MyKind> {
//!     TokenKindIdBinding { id: Number::kind_id(), value: MyKind::Number(self) }
//!   }
//! }
//! ```
//!
//! You should only use sub token kind structs to create [`TokenKindIdBinding`] to make sure the binding is correct.
//!
//! Besides, creating sub token kind structs is also helpful for the lexer implementation:
//! - In [`crate::lexer::action::Action::select`] we will use the sub token kind to ensure the action can only yield
//! one kind of token. And we can get the token kind id without executing the action.
//! - In expectational lexing, we can use the sub token kind type to get the expected token kind id,
//! without constructing a token kind value.
//!
//! To simplify all above, you can use the macro [`token_kind`] to transform the enum.
//!
//! ```
//! use whitehole::lexer::token::token_kind;
//! #[token_kind]
//! pub enum MyKind {
//!   Identifier(String),
//!   Number(i32),
//! }
//! # fn main() {}
//! ```
//!
//! Thats all you need to do, neat!

mod binding;
mod mock;
mod sub_token_kind;
mod token;
mod token_kind_id;

pub use binding::*;
pub use mock::*;
pub use sub_token_kind::*;
pub use token::*;
pub use token_kind_id::*;
pub use whitehole_macros::token_kind;

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole_macros::_token_kind;

  #[_token_kind]
  #[derive(Debug, Clone, Default)]
  pub enum MyKind {
    #[default]
    Unit,
    Unnamed(i32),
    Named {
      name: i32,
    },
  }

  #[test]
  fn token_kind_macro() {
    // generated structs
    let _ = Unit;
    Unnamed(42);
    Named { name: 42 };

    // unit variant is still unit variant instead of an unnamed variant
    let _ = MyKind::Unit;

    // other variants are transformed into unnamed variants
    MyKind::Unnamed(Unnamed(42));
    MyKind::Named(Named { name: 42 });

    // sub token kinds into token kind
    assert!(matches!(Unit.into(), MyKind::Unit));
    assert!(matches!(Unnamed(42).into(), MyKind::Unnamed(Unnamed(42))));
    assert!(matches!(
      Named { name: 42 }.into(),
      MyKind::Named(Named { name: 42 })
    ));

    // generated token kind id, as sub token kind
    assert_eq!(Unit::kind_id(), &TokenKindId::new(0));
    assert_eq!(Unnamed::kind_id(), &TokenKindId::new(1));
    assert_eq!(Named::kind_id(), &TokenKindId::new(2));

    // into token kind id binding
    let b: TokenKindIdBinding<MyKind> = Unit.into();
    assert_eq!(b.id(), Unit::kind_id());
    assert!(matches!(b.value(), MyKind::Unit));
    let b: TokenKindIdBinding<MyKind> = Unnamed(42).into();
    assert_eq!(b.id(), Unnamed::kind_id());
    assert!(matches!(b.value(), MyKind::Unnamed(Unnamed(42))));
    let b: TokenKindIdBinding<MyKind> = Named { name: 42 }.into();
    assert_eq!(b.id(), Named::kind_id());
    assert!(matches!(b.value(), MyKind::Named(Named { name: 42 })));

    // attributes are inherited by generated structs, e.g. Clone
    let _ = Unit.clone();
    let _ = Unnamed(42).clone();
    let _ = Named { name: 42 }.clone();

    // default is working
    assert!(matches!(MyKind::default(), MyKind::Unit));
    assert_eq!(MyKind::default_binding_kind_id(), Unit::kind_id());
  }
}
