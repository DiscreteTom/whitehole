//! ## Getting Started
//!
//! Here is the recommended order of learning this crate:
//!
//! - [`kind`]
//! - [`lexer`]

pub mod kind;
pub mod lexer;
pub mod utils;
// TODO: move parser in a standalone branch for now

pub mod combinator;
pub mod node;
pub mod parser;
