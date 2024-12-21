//! A parser combinator framework.
//!
//! # Getting Started
//!
//! Here is the recommended order of learning this project:
//!
//! - [`parse`] (optional)
//! - [`combinator`]
//! - [`parser`]
//!
//! # Related
//!
//! - [`in_str`](https://github.com/DiscreteTom/in_str/):
//!   A procedural macro to generate a closure that checks
//!   if a character is in the provided literal string.

pub mod combinator;
pub mod parse;
pub mod parser;
pub mod with_range;
