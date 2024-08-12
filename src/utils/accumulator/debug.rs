use super::Accumulator;
use std::fmt::Debug;

/// Print values using `println!("{:?}")` for debugging.
#[derive(Default, Debug, Clone, Copy)]
pub struct DebugAccumulator;
impl<T: Debug> Accumulator<T> for DebugAccumulator {
  #[inline]
  fn update(&mut self, t: T) {
    println!("{:?}", t);
  }
}
