//! ## For Developers
//!
//! Here is the recommended order of reading the source code:
//!
//! 1. [`self::token`]
//! 2. [`self::action`]
//! 4. [`self::state`]
//! 5. [`self::lexer`]
//! 6. [`self::builder`]
//! 3. [`self::stateless`]

pub mod action;
pub mod builder;
pub mod expectation;
pub mod fork;
pub mod lexer;
pub mod options;
pub mod output;
pub mod position;
pub mod re_lex;
pub mod state;
pub mod stateless;
pub mod token;

pub use builder::LexerBuilder;
pub use lexer::Lexer;

// TODO: organize exports
