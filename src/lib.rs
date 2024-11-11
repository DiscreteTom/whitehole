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
pub mod polyfill;

pub use whitehole_macros::in_str;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_in_str() {
    // simple case
    assert!(in_str!("123")('1'));
    assert!(in_str!("123")('2'));
    assert!(in_str!("123")('3'));
    // with escape
    assert!(in_str!("\n\r\t")('\n'));
    assert!(in_str!("\n\r\t")('\r'));
    assert!(in_str!("\n\r\t")('\t'));
    // with code point
    assert!(in_str!("\u{1F600}\u{10ffff}")('\u{1F600}'));
    assert!(in_str!("\u{1F600}\u{10ffff}")('\u{10ffff}'));
  }
}
