//! # Lookup Table
//!
//! This is the core module for the lexer and the parser.
//! We use lookup tables to replace hash maps as much as possible
//! to increase the runtime performance.
//!
//! # Getting Started
//!
//! Here is the recommended order of learning this module:
//!
//! - [`self`]
//! - [`self::option`]
//! - [`self::offset`]
//! - [`self::char`]
//!
//! // TODO: maybe publish this mod as a separate crate?

pub mod char;
pub mod offset;
pub mod option;

/// A trait for a lookup table.
pub(crate) trait Lookup {
  type Value;

  /// Return the value associated with the key.
  /// Return [`None`] if the key is not found or out of range.
  fn get(&self, key: usize) -> Option<&Self::Value>;

  /// Return the length of the table.
  /// This may not be the allocated size of the table.
  /// If a `key` is equal to or greater than the length,
  /// [`Lookup::get`] will always return [`None`].
  fn len(&self) -> usize;

  /// Return the mutable reference to the value associated with the key.
  /// # Safety
  /// This method is unsafe because it doesn't check whether the key is out of range
  /// or not found.
  ///
  /// [`debug_assert`] is used to check if the key is in range and valid.
  unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut Self::Value;
}
