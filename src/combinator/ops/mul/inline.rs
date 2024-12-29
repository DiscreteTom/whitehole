use super::{Mul, Repeat, Sep};
use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
};
use std::ops;

impl<
    Lhs: Action,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc) -> Acc,
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
    InlineFolder: Fn(T::Value, Acc) -> Acc,
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
    InlineFolder: Fn(Lhs::Value, Acc) -> Acc,
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

    let mut repeated = 0;
    let mut output = Output {
      value: init(),
      digested: 0,
    };

    while unsafe { repeat.validate(repeated) } {
      let Some(next_output) = (output.digested < input.rest().len())
        .then(|| unsafe { input.shift_unchecked(output.digested) })
        .and_then(|input| self.lhs.exec(input))
      else {
        break;
      };

      output.value = fold(next_output.value, output.value);
      repeated += 1;
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - output.digested > next_output.digested);
      output.digested = unsafe { output.digested.unchecked_add(next_output.digested) };
    }

    repeat.accept(repeated).then_some(output)
  }
}

unsafe impl<
    T: Action,
    S: Action<State = T::State, Heap = T::Heap>,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(T::Value, Acc) -> Acc,
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

    let mut repeated = 0;
    let mut output = Output {
      value: init(),
      digested: 0,
    };

    let mut digested_with_sep = 0;
    while unsafe { repeat.validate(repeated) } {
      let Some(value_output) = (digested_with_sep < input.rest().len())
        .then(|| unsafe { input.shift_unchecked(digested_with_sep) })
        .and_then(|input| self.lhs.value.exec(input))
      else {
        break;
      };
      repeated += 1;
      output.value = fold(value_output.value, output.value);
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - digested_with_sep > value_output.digested);
      output.digested = unsafe { digested_with_sep.unchecked_add(value_output.digested) };

      let Some(sep_output) = (output.digested < input.rest().len())
        .then(|| unsafe { input.shift_unchecked(output.digested) })
        .and_then(|input| self.lhs.sep.exec(input))
      else {
        break;
      };
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - output.digested > sep_output.digested);
      digested_with_sep = unsafe { output.digested.unchecked_add(sep_output.digested) };
    }

    repeat.accept(repeated).then_some(output)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::eat;

  #[test]
  fn test_inline_fold() {
    let combinator = eat('a').bind(1) * (1.., || 0, |v, acc| acc + v);
    let output = combinator
      .exec(Input::new("aaa", 0, &mut (), &mut ()).unwrap())
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 3);
  }

  #[test]
  fn test_inline_fold_with_sep() {
    let combinator = eat('a').bind(1).sep(',') * (1.., || 0, |v, acc| acc + v);
    let output = combinator
      .exec(Input::new("a,a,a", 0, &mut (), &mut ()).unwrap())
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 5);
  }
}
