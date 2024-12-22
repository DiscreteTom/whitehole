use super::{Mul, Repeat};
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
  fn mul(self, rhs: (Repeater, Initializer, InlineFolder)) -> Self::Output {
    Self::Output::new(Mul::new(self.action, rhs))
  }
}

impl<
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
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Value>> {
    let (range, init, folder) = &self.rhs;
    let mut repeated = 0;
    let mut output = Output {
      value: init(),
      rest: input.rest(),
    };

    while range.validate(repeated) {
      let Some(next_output) = input
        .reload(output.rest)
        .and_then(|mut input| self.lhs.exec(&mut input))
      else {
        break;
      };
      output.rest = next_output.rest;
      output.value = folder(next_output.value, output.value);
      repeated += 1;
    }

    range.accept(repeated).then_some(output)
  }
}
