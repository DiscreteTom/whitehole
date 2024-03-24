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
//! when the lexing is done we should already know the evaluated value of the string literal,
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
//! but they are the same kind of token.
//! To solve this problem, we can use a [`TokenKindId`] to identify different token kinds.
//! `Number(0)` and `Number(1)` are different values but they have the same [`TokenKindId`].
//! We use the index of the enum variant as the id of the token kind, so in the example above,
//! `Identifier` has id `0` and `Number` has id `1`.
//!
//! But we need to store the id and the token kind value together,
//! so we need [`TokenKindIdBinding`] to bind the id and the value.
//!
//! ```
//! # pub enum MyKind {
//! #   Identifier(String),
//! #   Number(i32),
//! # }
//! #
//! pub struct TokenKindIdBinding<TokenKindType> {
//!   id: TokenKindId<Self>,
//!   value: TokenKindType,
//! }
//!
//! // when creating `TokenKindIdBinding`, we have to make sure
//! // the id and the value are bound correctly.
//!
//! // correct
//! TokenKindIdBinding {
//!   id: TokenKindId::new(0),
//!   value: MyKind::Identifier("hello".to_string())
//! }
//! TokenKindIdBinding {
//!   id: TokenKindId::new(1),
//!   value: MyKind::Number(0)
//! }
//!
//! // wrong
//! TokenKindIdBinding {
//!   id: TokenKindId::new(0),
//!   value: MyKind::Number(0)
//! }
//! ```
//!
//! As you can see, we want to get the id bound with `MyKind`, and we get the id from [`TokenKindIdBinding`].
//! Thus the [`TokenKindIdBinding`] is a [`TokenKindIdProvider`], `MyKind` is not.
//!
//! To achieve the strict binding between the token kind id and the token kind value,
//! we will create structs for each enum variant and implement `Into<TokenKindIdBinding<MyKind>>` for them.
//!
//! ```
//! # use whitehole::lexer::token::{TokenKindId, TokenKindIdBinding};
//! #
//! // this is the "token kind"
//! pub enum MyKind {
//!   // instead of storing the value directly,
//!   // we store sub token kinds in the enum variant
//!   // to avoid destructing sub token kind value
//!   // when build the token kind value
//!   Identifier(Identifier),
//!   Number(Number),
//! }
//!
//! // these are "sub token kind"s, they store values
//! pub struct Identifier(pub String);
//! pub struct Number(pub i32);
//!
//! // every sub token kind should have a unique id
//! // bound with the type, not its value
//! impl SubTokenKind<TokenKindIdBinding<MyKind>> for Identifier {
//!   pub fn kind_id() -> TokenKindId<TokenKindIdBinding<MyKind>> {
//!     TokenKindId::new(0)
//!   }
//! }
//! impl SubTokenKind<TokenKindIdBinding<MyKind>> for Number {
//!   pub fn kind_id() -> TokenKindId<TokenKindIdBinding<MyKind>> {
//!     TokenKindId::new(1)
//!   }
//! }
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
//! // from sub token kinds we can create the token kind id bindings
//! impl Into<TokenKindIdBinding<MyKind>> for Identifier {
//!   fn into(self) -> TokenKindIdBinding<MyKind> {
//!     TokenKindIdBinding::new(self)
//!   }
//! }
//! impl Into<TokenKindIdBinding<MyKind>> for Number {
//!   fn into(self) -> TokenKindIdBinding<MyKind> {
//!     TokenKindIdBinding::new(self)
//!   }
//! }
//! ```
//!
//! We should only use these structs to create [`TokenKindIdBinding`] to make sure the binding is correct.
//!
//! To simplify all above, we can use the macro [`token_kind`] to transform the enum.
//!
//! ```
//! use whitehole::lexer::token::token_kind;
//! #[token_kind]
//! pub enum MyKind {
//!   Identifier(String),
//!   Number(i32),
//! }
//! ```
//!
//! Thats all we need to do, neat!

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
