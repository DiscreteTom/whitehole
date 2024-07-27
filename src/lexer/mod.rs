//! ## For Developers
//!
//! Here is the recommended order of reading the source code:
//!
//! 1. [`self::token`]
//! 2. [`self::action`]
//! 3. [`self::state`]
//! 4. [`self::expectation`]
//! 5. [`self::re_lex`]
//! 6. [`self::fork`]
//! 7. [`self::options`]
//! 8. [`self::output`]
//! 9. [`self::lexer`]
//! 10. [`self::builder`]
//! 11. [`self::stateless`]
//! 12. [`self::position`]

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
