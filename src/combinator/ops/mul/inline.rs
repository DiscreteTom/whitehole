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
  > ops::Mul<(Repeater, Initializer, InlineFolder)> for Combinator<Sep<T, S>>
{
  type Output = Combinator<Mul<Sep<T, S>, (Repeater, Initializer, InlineFolder)>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  fn mul(self, rhs: (Repeater, Initializer, InlineFolder)) -> Self::Output {
    Self::Output::new(Mul::new(self.action, rhs))
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
  fn exec<'text>(
    &self,
    mut input: Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let (repeat, init, fold) = &self.rhs;

    if !unsafe { repeat.validate(0) } {
      return repeat.accept(0).then(|| Output {
        value: init(),
        digested: 0,
      });
    }

    // the first occurrence
    let (mut repeated, mut output) = if let Some(output) = self.lhs.exec(input.reborrow()) {
      (1, output.map(|value| fold(value, init())))
    } else {
      return repeat.accept(0).then(|| Output {
        value: init(),
        digested: 0,
      });
    };

    // the rest of the occurrences
    while unsafe { repeat.validate(repeated) } {
      let Some(new_output) = (output.digested < input.rest().len())
        .then(|| unsafe { input.shift_unchecked(output.digested) })
        .and_then(|input| self.lhs.exec(input))
      else {
        break;
      };

      repeated += 1;
      {
        // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
        debug_assert!(usize::MAX - output.digested > new_output.digested);
        output.digested = unsafe { output.digested.unchecked_add(new_output.digested) };
      }
      output.value = fold(new_output.value, output.value);
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
  fn exec<'text>(
    &self,
    mut input: Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let (repeat, init, fold) = &self.rhs;

    if !unsafe { repeat.validate(0) } {
      return repeat.accept(0).then(|| Output {
        value: init(),
        digested: 0,
      });
    }

    // the first occurrence of `value`
    let (mut repeated, mut output) = if let Some(output) = self.lhs.value.exec(input.reborrow()) {
      (1, output.map(|value| fold(value, init())))
    } else {
      return repeat.accept(0).then(|| Output {
        value: init(),
        digested: 0,
      });
    };

    // the rest of the occurrences
    while unsafe { repeat.validate(repeated) } {
      let Some(sep_output) = (output.digested < input.rest().len())
        .then(|| unsafe { input.shift_unchecked(output.digested) })
        .and_then(|input| self.lhs.sep.exec(input))
      else {
        break;
      };
      let digested_with_sep = {
        // SAFETY: since `slice::len` is usize, so `digested_with_sep` must be a valid usize
        debug_assert!(usize::MAX - output.digested > sep_output.digested);
        unsafe { output.digested.unchecked_add(sep_output.digested) }
      };
      let Some(value_output) = (digested_with_sep < input.rest().len())
        .then(|| unsafe { input.shift_unchecked(digested_with_sep) })
        .and_then(|input| self.lhs.value.exec(input))
      else {
        break;
      };

      // now we have both `value` and `sep`, update `output` and `repeated`
      repeated += 1;
      {
        // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
        debug_assert!(usize::MAX - digested_with_sep > sep_output.digested);
        output.digested = unsafe { digested_with_sep.unchecked_add(value_output.digested) };
      }
      output.value = fold(value_output.value, output.value);
    }

    repeat.accept(repeated).then_some(output)
  }
}
