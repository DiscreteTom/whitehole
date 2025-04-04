//! Operator overloading for [`Combinator`](crate::combinator::Combinator).

mod compose;

pub use compose::*;

pub mod add;
pub mod bitor;
pub mod mul;
pub mod not;
