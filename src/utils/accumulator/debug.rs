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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_debug_accumulator() {
    // TODO: how to capture the stdout?
    let mut acc = DebugAccumulator;
    acc.update(1);
    acc.update("hello");
    acc.update(vec![1, 2, 3]);
  }
}
