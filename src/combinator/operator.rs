use super::Combinator;
use std::ops;

impl<'a, Kind: 'a, State: 'a, Heap: 'a> ops::BitOr for Combinator<'a, Kind, State, Heap> {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self::Output {
    Combinator {
      exec: Box::new(move |input| self.parse(input).or_else(|| rhs.parse(input))),
    }
  }
}
