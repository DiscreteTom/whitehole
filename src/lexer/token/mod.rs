//! ## Design
//!
//! Usually when you want to write a lexer, you need to define "token kinds",
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
//! Besides, you may want to carry some data with different token kinds.
//! The data may be generated during the lexing process and stored in the token
//! so you don't need to parse the token content again after lexing.
//! An example is that if you want to lex a string literal with escape sequences,
//! when the token is yielded you should already know the evaluated value of the string literal,
//! you can store the value in the token, instead of parsing the literal content again.
//! The data should be associated with the token kind,
//! so you can use enum variants to represent them.
//!
//! ```
//! pub enum MyKind {
//!   Identifier(String),
//!   Number(i32),
//! }
//! ```
//!
//! However, in rust `Number(0)` and `Number(1)` are different values,
//! but their token kinds are the same.
//! To solve this problem, we define [`TokenKindId`] to identify different token kinds.
//! `Number(0)` and `Number(1)` are different values but they have the same [`TokenKindId`].
//! The value of the [`TokenKindId`] is the index of the enum variant,
//! so the [`TokenKindId`] is unique for each variant.
//!
//! You also need a way to get the token kind id from a token kind value.
//! An easy way is to use pattern matching like this:
//!
//! ```
//! # pub enum MyKind {
//! #   Identifier(String),
//! #   Number(i32),
//! # }
//! pub struct TokenKindId(usize);
//! fn get_id(kind: &MyKind) -> TokenKindId {
//!   match kind {
//!     MyKind::Identifier(_) => TokenKindId(0),
//!     MyKind::Number(_) => TokenKindId(1),
//!   }
//! }
//! ```
//!
//! However we will access the token kind id frequently, so we store the id and the token kind value together
//! to prevent unnecessary pattern matching (just like cache the result of the pattern matching).
//! We use [`TokenKindIdBinding`] to bind the id and the value.
//!
//! ```
//! # use std::marker::PhantomData;
//! # pub struct TokenKindId<T>(usize, PhantomData<T>);
//! # impl<T> TokenKindId<T> {
//! #   pub fn new(id: usize) -> Self {
//! #     Self(id, PhantomData)
//! #   }
//! # }
//! # pub enum MyKind {
//! #   Identifier(String),
//! #   Number(i32),
//! # }
//! #
//! pub struct TokenKindIdBinding<TokenKindType> {
//!   id: TokenKindId<TokenKindType>,
//!   kind: TokenKindType,
//! };
//!
//! // when creating `TokenKindIdBinding`, we have to make sure
//! // the id and the kind are bound correctly.
//!
//! // correct
//! TokenKindIdBinding {
//!   id: TokenKindId::new(0), // the id of `Identifier`
//!   kind: MyKind::Identifier("hello".to_string())
//! };
//! TokenKindIdBinding {
//!   id: TokenKindId::new(1), // the id of `Number`
//!   kind: MyKind::Number(0)
//! };
//!
//! // wrong!
//! TokenKindIdBinding {
//!   id: TokenKindId::new(0), // the id of `Identifier`
//!   kind: MyKind::Number(0)
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
//! # use std::marker::PhantomData;
//! # pub struct TokenKindId<T>(usize, PhantomData<T>);
//! # impl<T> TokenKindId<T> {
//! #   pub fn new(id: usize) -> Self {
//! #     Self(id, PhantomData)
//! #   }
//! # }
//! # pub struct TokenKindIdBinding<TokenKindType> {
//! #   id: TokenKindId<TokenKindType>,
//! #   kind: TokenKindType,
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
//! impl SubTokenKind<MyKind> for Identifier {
//!   fn kind_id() -> TokenKindId<MyKind> {
//!     TokenKindId::new(0)
//!   }
//! }
//! impl SubTokenKind<MyKind> for Number {
//!   fn kind_id() -> TokenKindId<MyKind> {
//!     TokenKindId::new(1)
//!   }
//! }
//!
//! // from sub token kinds we can create the token kind id bindings
//! impl Into<TokenKindIdBinding<MyKind>> for Identifier {
//!   fn into(self) -> TokenKindIdBinding<MyKind> {
//!     TokenKindIdBinding { id: Identifier::kind_id(), kind: MyKind::Identifier(self) }
//!   }
//! }
//! impl Into<TokenKindIdBinding<MyKind>> for Number {
//!   fn into(self) -> TokenKindIdBinding<MyKind> {
//!     TokenKindIdBinding { id: Number::kind_id(), kind: MyKind::Number(self) }
//!   }
//! }
//! ```
//!
//! You should only use sub token kind structs to create [`TokenKindIdBinding`] to make sure the binding is correct.
//!
//! Besides, creating sub token kind structs is also helpful for the lexer implementation:
//! - In [`crate::lexer::action::Action::select`] we will use the sub token kind to ensure the action can only yield
//! one kind of token. And we can infer [`crate::lexer::action::Action::kind`] statically without executing the action.
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
//!
//! ## For Developers
//!
//! Here is the recommended order of reading the source code:
//!
//! - [`self::token_kind_id`]
//! - [`self::sub_token_kind`]
//! - [`self::binding`]
//! - [`self::mock`]
//! - [`self::token`]
//! - [`self::token_kind`]
//!
//! The [`token_kind`] macro will be tested in this file.

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
  #[derive(Debug, Clone, Default, PartialEq, Eq)]
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

    // into token kind id binding
    let b: TokenKindIdBinding<MyKind> = Unit.into();
    assert_eq!(b.id(), Unit::kind_id());
    assert_eq!(b.take(), MyKind::Unit);
    let b: TokenKindIdBinding<MyKind> = Unnamed(42).into();
    assert_eq!(b.id(), Unnamed::kind_id());
    assert_eq!(b.take(), MyKind::Unnamed(Unnamed(42)));
    let b: TokenKindIdBinding<MyKind> = Named { name: 42 }.into();
    assert_eq!(b.id(), Named::kind_id());
    assert_eq!(b.take(), MyKind::Named(Named { name: 42 }));

    // generated token kind id, as sub token kind.
    let v: Vec<TokenKindId<MyKind>> = vec![Unit::kind_id(), Unnamed::kind_id(), Named::kind_id()];
    for (i, id) in v.iter().enumerate() {
      for (j, id2) in v.iter().enumerate() {
        if i == j {
          assert_eq!(id, id2);
        } else {
          assert_ne!(id, id2);
        }
      }
    }

    // sub token kind into token kind id
    assert_eq!(
      <Unit as Into<TokenKindId<MyKind>>>::into(Unit),
      Unit::kind_id()
    );
    assert_eq!(
      <Unnamed as Into<TokenKindId<MyKind>>>::into(Unnamed(42)),
      Unnamed::kind_id()
    );
    assert_eq!(
      <Named as Into<TokenKindId<MyKind>>>::into(Named { name: 42 }),
      Named::kind_id()
    );

    // attributes are inherited by generated structs, e.g. Clone
    let _ = Unit.clone();
    let _ = Unnamed(42).clone();
    let _ = Named { name: 42 }.clone();
    let _ = MyKind::Unit.clone();
    let _ = MyKind::Unnamed(Unnamed(42)).clone();
    let _ = MyKind::Named(Named { name: 42 }).clone();

    // default is working
    assert!(matches!(MyKind::default(), MyKind::Unit));
    assert_eq!(MyKind::default_kind_id(), Unit::kind_id());
  }
}
