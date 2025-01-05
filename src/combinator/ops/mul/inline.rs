use super::{impl_mul, impl_mul_with_sep, Mul, Repeat, Sep};
use crate::{
  action::{shift_input, Action, Input, Output},
  combinator::Combinator,
};
use std::ops;

impl<
    Lhs: Action,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc, Input<&mut Lhs::State, &mut Lhs::Heap>) -> Acc,
  > ops::Mul<(Repeater, Initializer, InlineFolder)> for Combinator<Lhs>
{
  type Output = Combinator<Mul<Lhs, (Repeater, Initializer, InlineFolder)>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  fn mul(self, rhs: (Repeater, Initializer, InlineFolder)) -> Self::Output {
    Self::Output::new(Mul::new(self.action, rhs))
  }
}

impl<
    T: Action,
    S: Action<State = T::State, Heap = T::Heap>,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(T::Value, Acc, Input<&mut T::State, &mut T::Heap>) -> Acc,
  > ops::Mul<(Repeater, Initializer, InlineFolder)> for Sep<T, S>
{
  type Output = Combinator<Mul<Sep<T, S>, (Repeater, Initializer, InlineFolder)>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  fn mul(self, rhs: (Repeater, Initializer, InlineFolder)) -> Self::Output {
    Self::Output::new(Mul::new(self, rhs))
  }
}

unsafe impl<
    Lhs: Action,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc, Input<&mut Lhs::State, &mut Lhs::Heap>) -> Acc,
  > Action for Mul<Lhs, (Repeater, Initializer, InlineFolder)>
{
  type Value = Acc;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let (repeat, init, fold) = &self.rhs;
    impl_mul!(input, repeat, init, fold, self.lhs)
  }
}

unsafe impl<
    T: Action,
    S: Action<State = T::State, Heap = T::Heap>,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(T::Value, Acc, Input<&mut T::State, &mut T::Heap>) -> Acc,
  > Action for Mul<Sep<T, S>, (Repeater, Initializer, InlineFolder)>
{
  type Value = Acc;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let (repeat, init, fold) = &self.rhs;
    impl_mul_with_sep!(input, repeat, init, fold, self.lhs.value, self.lhs.sep)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::eat, instant::Instant};

  #[test]
  fn test_inline_fold() {
    let combinator = eat('a').bind(1) * (1.., || 0, |v, acc, _| acc + v);
    let output = combinator
      .exec(Input::new(Instant::new("aaa"), &mut (), &mut ()).unwrap())
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 3);
  }

  #[test]
  fn test_inline_fold_with_sep() {
    let combinator = eat('a').bind(1).sep(',') * (1.., || 0, |v, acc, _| acc + v);
    let output = combinator
      .exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ()).unwrap())
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 5);
  }
}
