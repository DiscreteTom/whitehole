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

    if !repeat.validate(0) {
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
    while repeat.validate(repeated) {
      let Some(new_output) = input
        .reload(output.digested)
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
