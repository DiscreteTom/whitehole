mod debug;
mod mock;
mod setter;
mod string;
mod vec;

pub use debug::*;
pub use setter::*;

/// Accumulate values.
///
/// These types already implement the [`Accumulator`] trait:
/// - `()` - mock accumulator that does nothing.
/// - [`Vec<T>`] - accumulate values into a vector.
/// - [`String`] - accumulate characters or strings into a string.
/// - [`DebugAccumulator`] - print values to stdout.
pub trait Accumulator<T> {
  /// Update the accumulator with a value.
  fn update(&mut self, t: T);
}

// if a type implements `Accumulator<T>`, then its mutable reference should also implement it
impl<Acc: Accumulator<T>, T> Accumulator<T> for &mut Acc {
  #[inline]
  fn update(&mut self, c: T) {
    <Acc as Accumulator<T>>::update(*self, c);
  }
}
