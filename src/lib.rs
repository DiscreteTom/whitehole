//! A parser combinator framework.
//!
//! # Getting Started
//!
//! Here is the recommended order of learning this project:
//!
//! - [`action`] (optional)
//! - [`combinator`]
//! - [`parser`]
//!
//! # Related
//!
//! - [`in_str`](https://github.com/DiscreteTom/in_str/):
//!   A procedural macro to generate a closure that checks
//!   if a character is in the provided literal string.

pub mod action;
pub mod combinator;
pub mod parser;
pub mod range;
