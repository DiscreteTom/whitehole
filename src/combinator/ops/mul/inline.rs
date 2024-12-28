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
      let Some(new_output) = input
        .shift(output.digested)
        .and_then(|input| self.lhs.exec(input))
      else {
        break;
      };

      repeated += 1;
      output.digested += new_output.digested;
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
      let Some(sep_output) = input
        .shift(output.digested)
        .and_then(|input| self.lhs.sep.exec(input))
      else {
        break;
      };
      let Some(value_output) = input
        .shift(output.digested + sep_output.digested)
        .and_then(|input| self.lhs.value.exec(input))
      else {
        break;
      };

      // now we have both `value` and `sep`, update `output` and `repeated`
      repeated += 1;
      output.digested += sep_output.digested + value_output.digested;
      output.value = fold(value_output.value, output.value);
    }

    repeat.accept(repeated).then_some(output)
  }
}
