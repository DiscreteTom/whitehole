use super::{impl_mul, impl_mul_with_sep, Mul, Repeat, Sep};
use crate::{
  action::{shift_input, Action, Input, Output},
  combinator::Combinator,
};
use std::ops;

impl<Lhs, Acc, Repeater: Repeat, Initializer: Fn() -> Acc, InlineFolder>
  ops::Mul<(Repeater, Initializer, InlineFolder)> for Combinator<Lhs>
{
  type Output = Combinator<Mul<Combinator<Lhs>, (Repeater, Initializer, InlineFolder)>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  fn mul(self, rhs: (Repeater, Initializer, InlineFolder)) -> Self::Output {
    Self::Output::new(Mul::new(self, rhs))
  }
}

impl<T, S, Acc, Repeater: Repeat, Initializer: Fn() -> Acc, InlineFolder>
  ops::Mul<(Repeater, Initializer, InlineFolder)> for Sep<T, S>
{
  type Output = Combinator<Mul<Sep<T, S>, (Repeater, Initializer, InlineFolder)>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  fn mul(self, rhs: (Repeater, Initializer, InlineFolder)) -> Self::Output {
    Self::Output::new(Mul::new(self, rhs))
  }
}

unsafe impl<
    State,
    Heap,
    Lhs: Action<State, Heap>,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc, Input<&mut State, &mut Heap>) -> Acc,
  > Action<State, Heap> for Mul<Combinator<Lhs>, (Repeater, Initializer, InlineFolder)>
{
  type Value = Acc;

  #[inline]
  fn exec(&self, mut input: Input<&mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    let (repeat, init, fold) = &self.rhs;
    impl_mul!(input, repeat, init, fold, self.lhs)
  }
}

unsafe impl<
    State,
    Heap,
    T: Action<State, Heap>,
    S: Action<State, Heap>,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(T::Value, Acc, Input<&mut State, &mut Heap>) -> Acc,
  > Action<State, Heap> for Mul<Sep<T, S>, (Repeater, Initializer, InlineFolder)>
{
  type Value = Acc;

  #[inline]
  fn exec(&self, mut input: Input<&mut State, &mut Heap>) -> Option<Output<Self::Value>> {
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
    // TODO: find a way to remove the signature of `Input`. See https://users.rust-lang.org/t/implementation-of-fnonce-is-not-general-enough/68294/1
    let combinator = eat('a').bind(1) * (1.., || 0, |v, acc, _: Input<&mut (), &mut ()>| acc + v);
    let output = combinator
      .exec(Input::new(Instant::new("aaa"), &mut (), &mut ()).unwrap())
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 3);
  }

  #[test]
  fn test_inline_fold_with_sep() {
    let combinator = eat('a').bind(1).sep(',')
      * (
        1..,
        || 0,
        |v, acc, _: Input<&mut (), &mut ()>| acc + v | acc + v,
      );
    let output = combinator
      .exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ()).unwrap())
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 5);
  }
}
