//! # Lexer
//!
//! ## For Developers
//!
//! Here is the recommended order of reading:
//!
//! 1. [`self::token`]
//! 2. [`self::action`]
//! 3. [`self::stateless`]
//! 4. [`self::state`]
//! 5. [`self::lexer`]

pub mod action;
pub mod builder;
pub mod expectation;
pub mod lexer;
pub mod options;
pub mod output;
pub mod position;
pub mod state;
pub mod stateless;
pub mod token;

pub use action::Action;
pub use builder::LexerBuilder;
pub use lexer::Lexer;

// TODO: organize exports
