//! # Kind Management
//!
//! ## Design
//!
//! Usually when you want to write a lexer, you need to define different variants of tokens,
//! like `Identifier`, `Number`, etc.
//! You can use enum to represent these different variants of tokens.
//!
//! ```
//! // we call the enum "kind", a group of "sub kinds"
//! pub enum MyKind {
//!   // each variant represents a "sub kind"
//!   Identifier,
//!   Number,
//! }
//! // then you can have something like `Token<MyKind>`
//! ```
//!
//! Besides, you may want to carry some data with different sub kinds.
//! The data may be generated during the lexing process and stored in the sub kind,
//! so you don't need to parse the token content again after lexing.
//! An example is that if you want to lex a string literal with escape sequences,
//! when the token is yielded you should already know the evaluated value of the string literal,
//! you can store the value in the sub kind, instead of parsing the token's literal content again.
//! The data should be associated with the sub kind instead of the kind,
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
//! but their sub kinds are the same.
//! To solve this problem, we define [`SubKindId`] to identify different sub kinds.
//! `Number(0)` and `Number(1)` are different values but they have the same [`SubKindId`].
//! The value of the [`SubKindId`] is the index of the enum variant,
//! so the [`SubKindId`] is unique for each variant.
//!
//! You also need a way to get the sub kind id from a kind value.
//! An easy way is to use pattern matching like this:
//!
//! ```
//! # pub enum MyKind {
//! #   Identifier(String),
//! #   Number(i32),
//! # }
//! pub struct SubKindId(usize);
//! fn get_id(kind: &MyKind) -> SubKindId {
//!   match kind {
//!     MyKind::Identifier(_) => SubKindId(0),
//!     MyKind::Number(_) => SubKindId(1),
//!   }
//! }
//! ```
//!
//! However we will access the sub kind id frequently, so we store the id and the kind value together
//! to prevent unnecessary pattern matching (just like cache the result of the pattern matching).
//! We use [`KindIdBinding`] to bind the sub kind id and the kind value.
//!
//! ```
//! # use std::marker::PhantomData;
//! # pub struct SubKindId<T>(usize, PhantomData<T>);
//! # impl<T> SubKindId<T> {
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
//!   id: SubKindId<Kind>,
//!   kind: Kind,
//! };
//!
//! // when creating `KindIdBinding`, you have to make sure
//! // the id and the kind are bound correctly.
//!
//! // correct
//! KindIdBinding {
//!   id: SubKindId::new(0), // the id of `Identifier`
//!   kind: MyKind::Identifier("hello".to_string())
//! };
//! KindIdBinding {
//!   id: SubKindId::new(1), // the id of `Number`
//!   kind: MyKind::Number(0)
//! };
//!
//! // wrong!
//! KindIdBinding {
//!   id: SubKindId::new(0), // the id of `Identifier`
//!   kind: MyKind::Number(0)
//! };
//! ```
//!
//! To achieve the strict binding between the sub kind id and the kind value,
//! we will create structs for each enum variant and implement `Into<KindIdBinding<MyKind>>` for them.
//!
//! ```
//! # use std::marker::PhantomData;
//! # pub struct SubKindId<T>(usize, PhantomData<T>);
//! # impl<T> SubKindId<T> {
//! #   pub fn new(id: usize) -> Self {
//! #     Self(id, PhantomData)
//! #   }
//! # }
//! # pub struct KindIdBinding<KindType> {
//! #   id: SubKindId<KindType>,
//! #   kind: KindType,
//! # };
//! pub enum MyKind {
//!   // instead of storing the data directly,
//!   // we store "sub kind" values in the enum variant
//!   // to avoid destructing sub kind values
//!   // when constructing kind values
//!   // TODO: is this faster?
//!   Identifier(Identifier),
//!   Number(Number),
//! }
//!
//! // "sub kinds" will store token's data
//! pub struct Identifier(pub String);
//! pub struct Number(pub i32);
//!
//! // sub kinds can be converted into the kind
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
//!   // every sub kind should have a unique id
//!   // bound with the type, not its value
//!   fn kind_id() -> SubKindId<Self::Kind>;
//! }
//!
//! impl SubKind for Identifier {
//!   type Kind = MyKind;
//!   fn kind_id() -> SubKindId<MyKind> {
//!     KindId::new(0)
//!   }
//! }
//! impl SubKind for Number {
//!   type Kind = MyKind;
//!   fn kind_id() -> KindId<MyKind> {
//!     SubKindId::new(1)
//!   }
//! }
//!
//! // from sub kinds we can create the kind id bindings
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
//! You should only use sub kind structs to create [`KindIdBinding`] to make sure the binding is correct.
//!
//! Besides, creating sub kind structs is also helpful for the lexer implementation:
//! - In [`crate::lexer::action::Action::select`] we will use the sub kind to ensure the action can only yield
//!   one kind of token. And we can infer [`crate::lexer::action::Action::kind`] statically without executing the action.
//! - In expectational lexing, you can use the sub kind type to get the expected kind id,
//!   without constructing a kind value.
//!
//! To simplify all above, you can use the macro [`whitehole_kind`] to transform the enum.
//!
//! ```
//! use whitehole::kind::whitehole_kind;
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

mod binding;
mod id;
mod mock;
mod sub;

pub use binding::*;
pub use id::*;
pub use mock::*;
pub use sub::*;
pub use whitehole_macros::whitehole_kind;

#[cfg(test)]
mod tests {
  use super::*;
  use crate::kind::{DefaultSubKind, SubKind, SubKindId};
  use whitehole_macros::_whitehole_kind;

  #[_whitehole_kind]
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
  fn kind_macro() {
    // generated structs
    let _ = Unit;
    let _ = Unnamed(42);
    let _ = Named { name: 42 };

    // unit variant is still unit variant instead of an unnamed variant
    let _ = MyKind::Unit;

    // other variants are transformed into unnamed variants
    let _ = MyKind::Unnamed(Unnamed(42));
    let _ = MyKind::Named(Named { name: 42 });

    // sub token kinds into token kind
    assert!(matches!(Unit.into(), MyKind::Unit));
    assert!(matches!(Unnamed(42).into(), MyKind::Unnamed(Unnamed(42))));
    assert!(matches!(
      Named { name: 42 }.into(),
      MyKind::Named(Named { name: 42 })
    ));

    // into token kind id binding
    let b: KindIdBinding<MyKind> = Unit.into();
    assert_eq!(b.id(), Unit::kind_id());
    assert_eq!(b.take(), MyKind::Unit);
    let b: KindIdBinding<MyKind> = Unnamed(42).into();
    assert_eq!(b.id(), Unnamed::kind_id());
    assert_eq!(b.take(), MyKind::Unnamed(Unnamed(42)));
    let b: KindIdBinding<MyKind> = Named { name: 42 }.into();
    assert_eq!(b.id(), Named::kind_id());
    assert_eq!(b.take(), MyKind::Named(Named { name: 42 }));

    // generated token kind id, as sub token kind.
    let v: Vec<SubKindId<MyKind>> = vec![Unit::kind_id(), Unnamed::kind_id(), Named::kind_id()];
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
      <Unit as Into<SubKindId<MyKind>>>::into(Unit),
      Unit::kind_id()
    );
    assert_eq!(
      <Unnamed as Into<SubKindId<MyKind>>>::into(Unnamed(42)),
      Unnamed::kind_id()
    );
    assert_eq!(
      <Named as Into<SubKindId<MyKind>>>::into(Named { name: 42 }),
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
