//! ## For Developers
//!
//! Here is the recommended order of reading the source code:
//!
//! - [`self::token`]
//! - [`self::action`]
//! - [`self::instant`]
//! - [`self::expectation`]
//! - [`self::re_lex`]
//! - [`self::fork`]
//! - [`self::options`]
//! - [`self::output`]
//! - [`self::lexer`]
//! - [`self::builder`]
//! - [`self::stateless`]
//! - [`self::position`]

pub mod action;
pub mod builder;
pub mod expectation;
pub mod fork;
pub mod instant;
pub mod lexer;
pub mod options;
pub mod output;
pub mod position;
pub mod re_lex;
pub mod stateless;
pub mod token;

pub use builder::LexerBuilder;
pub use lexer::Lexer;

// TODO: organize exports
