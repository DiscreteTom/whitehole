//! # Kind Management
//!
//! ## Design
//!
//! Usually when you want to write a lexer, you need to define "kinds" for tokens,
//! like `Identifier`, `Number`, etc.
//! You can use enum to represent these kinds.
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
//! To solve this problem, we define [`KindId`] to identify different token kinds.
//! `Number(0)` and `Number(1)` are different values but they have the same [`KindId`].
//! The value of the [`KindId`] is the index of the enum variant,
//! so the [`KindId`] is unique for each variant.
//!
//! You also need a way to get the token kind id from a token kind value.
//! An easy way is to use pattern matching like this:
//!
//! ```
//! # pub enum MyKind {
//! #   Identifier(String),
//! #   Number(i32),
//! # }
//! pub struct KindId(usize);
//! fn get_id(kind: &MyKind) -> KindId {
//!   match kind {
//!     MyKind::Identifier(_) => KindId(0),
//!     MyKind::Number(_) => KindId(1),
//!   }
//! }
//! ```
//!
//! However we will access the token kind id frequently, so we store the id and the token kind value together
//! to prevent unnecessary pattern matching (just like cache the result of the pattern matching).
//! We use [`KindIdBinding`] to bind the id and the value.
//!
//! ```
//! # use std::marker::PhantomData;
//! # pub struct KindId<T>(usize, PhantomData<T>);
//! # impl<T> KindId<T> {
//! #   pub fn new(id: usize) -> Self {
//! #     Self(id, PhantomData)
//! #   }
//! # }
//! # pub enum MyKind {
//! #   Identifier(String),
//! #   Number(i32),
//! # }
//! #
//! pub struct KindIdBinding<Kind> {
//!   id: KindId<Kind>,
//!   kind: Kind,
//! };
//!
//! // when creating `KindIdBinding`, you have to make sure
//! // the id and the kind are bound correctly.
//!
//! // correct
//! KindIdBinding {
//!   id: KindId::new(0), // the id of `Identifier`
//!   kind: MyKind::Identifier("hello".to_string())
//! };
//! KindIdBinding {
//!   id: KindId::new(1), // the id of `Number`
//!   kind: MyKind::Number(0)
//! };
//!
//! // wrong!
//! KindIdBinding {
//!   id: KindId::new(0), // the id of `Identifier`
//!   kind: MyKind::Number(0)
//! };
//! ```
//!
//! To achieve the strict binding between the token kind id and the token kind value,
//! we will create structs for each enum variant and implement `Into<KindIdBinding<MyKind>>` for them.
//!
//! ```
//! # use std::marker::PhantomData;
//! # pub struct KindId<T>(usize, PhantomData<T>);
//! # impl<T> KindId<T> {
//! #   pub fn new(id: usize) -> Self {
//! #     Self(id, PhantomData)
//! #   }
//! # }
//! # pub struct KindIdBinding<KindType> {
//! #   id: KindId<KindType>,
//! #   kind: KindType,
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
//! pub trait SubKind {
//!   type Kind;
//!   fn kind_id() -> KindId<Self::Kind>;
//! }
//!
//! // every sub token kind should have a unique id
//! // bound with the type, not its value
//! impl SubKind for Identifier {
//!   type Kind = MyKind;
//!   fn kind_id() -> KindId<MyKind> {
//!     KindId::new(0)
//!   }
//! }
//! impl SubKind for Number {
//!   type Kind = MyKind;
//!   fn kind_id() -> KindId<MyKind> {
//!     KindId::new(1)
//!   }
//! }
//!
//! // from sub token kinds we can create the token kind id bindings
//! impl Into<KindIdBinding<MyKind>> for Identifier {
//!   fn into(self) -> KindIdBinding<MyKind> {
//!     KindIdBinding { id: Identifier::kind_id(), kind: MyKind::Identifier(self) }
//!   }
//! }
//! impl Into<KindIdBinding<MyKind>> for Number {
//!   fn into(self) -> KindIdBinding<MyKind> {
//!     KindIdBinding { id: Number::kind_id(), kind: MyKind::Number(self) }
//!   }
//! }
//! ```
//!
//! You should only use sub token kind structs to create [`KindIdBinding`] to make sure the binding is correct.
//!
//! Besides, creating sub token kind structs is also helpful for the lexer implementation:
//! - In [`crate::lexer::action::Action::select`] we will use the sub token kind to ensure the action can only yield
//!   one kind of token. And we can infer [`crate::lexer::action::Action::kind`] statically without executing the action.
//! - In expectational lexing, you can use the sub token kind type to get the expected token kind id,
//!   without constructing a token kind value.
//!
//! To simplify all above, you can use the macro [`kind`] to transform the enum.
//!
//! ```
//! use whitehole::lexer::token::kind;
//! #[whitehole_kind]
//! pub enum MyKind {
//!   Identifier(String),
//!   Number(i32),
//! }
//! # fn main() {}
//! ```
//!
//! Thats all you need to do, neat!
//!
//! ## Getting Started
//!
//! Here is the recommended order of learning this module:
//!
//! - [`self::id`]
//! - [`self::sub`]
//! - [`self::binding`]
//! - [`self::mock`]
//! - [`self`]

mod binding;
mod id;
mod mock;
mod sub;

pub use binding::*;
pub use id::*;
pub use mock::*;
pub use sub::*;
pub use whitehole_macros::whitehole_kind;
