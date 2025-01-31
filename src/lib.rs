//! A simple, fast, intuitive parser combinator framework for Rust.
//!
//! # Get Started
//!
//! ```
//! use whitehole::{
//!   combinator::eat,
//!   parser::Parser,
//! };
//!
//! // define the kind of the output
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! enum Kind {
//!   A,
//! }
//!
//! let mut parser = Parser::builder() // create a parser builder
//!   .entry(eat("a").bind(Kind::A)) // set the entry action
//!   .build("a"); // build the parser with the input
//!
//! let output = parser.next().unwrap(); // yield the next output
//! assert_eq!(output.value, Kind::A); // check the output
//! ```
//!
//! See the [`combinator`] module to learn how to compose the entry action.
//! See the [`parser`] module to learn how to use the parser.
//!
//! # Read the Source Code
//!
//! Here is the recommended order to read the source code:
//!
//! - [`digest`]
//! - [`instant`]
//! - [`action`]
//! - [`combinator`]
//! - [`parser`]

pub mod action;
pub mod combinator;
pub mod digest;
pub mod instant;
pub mod parser;
pub mod range;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}
