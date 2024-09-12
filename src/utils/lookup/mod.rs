//! # Lookup Table
//!
//! This is the core module for the lexer and the parser.
//! We use lookup tables to replace hash maps as much as possible
//! to increase the runtime performance.
//!
//! Since the performance is critical, in this module
//! we will use a lot of unsafe code to bypass some runtime checks.
//!
//! # Getting Started
//!
//! Here is the recommended order of learning this module:
//!
//! - [`self::lookup`]
//! - [`self::option`]
//! - [`self::offset`]
//! - [`self::char`]
//!
//! // TODO: maybe publish this mod as a separate crate?

pub mod char;
pub mod lookup;
pub mod offset;
pub mod option;
