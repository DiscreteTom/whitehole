mod debug;
mod mock;
mod string;
mod vec;

pub use debug::*;

/// Accumulate values.
///
/// These types already implement the [`Accumulator`] trait:
/// - `()` - mock accumulator that does nothing.
/// - [`Vec<T>`] - accumulate values into a vector.
/// - [`String`] - accumulate characters or strings into a string.
pub trait Accumulator<T> {
  /// Update the accumulator with a value.
  fn update(&mut self, t: T);
}
