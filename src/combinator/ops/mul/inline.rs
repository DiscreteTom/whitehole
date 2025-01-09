use super::{impl_mul, impl_mul_with_sep, Mul, Repeat, Sep};
use crate::{
  action::{shift_input, Action, Input, Output},
  combinator::Combinator,
};
use core::fmt;
use std::ops;

/// See [`Combinator::fold`].
#[derive(Copy, Clone)]
pub struct InlineFold<T, Init, Folder> {
  action: T,
  init: Init,
  fold: Folder,
}

impl<T: fmt::Debug, Init, Folder> fmt::Debug for InlineFold<T, Init, Folder> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("InlineFold")
      .field("action", &self.action)
      .finish()
  }
}

impl<T> Combinator<T> {
  /// TODO: more comments
  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  pub fn fold<
    State,
    Heap,
    Acc,
    Init: Fn() -> Acc,
    Folder: Fn(T::Value, Acc, Input<&mut State, &mut Heap>) -> Acc,
  >(
    self,
    init: Init,
    folder: Folder,
  ) -> InlineFold<T, Init, Folder>
  where
    T: Action<State, Heap>,
  {
    InlineFold {
      action: self.action,
      init,
      fold: folder,
    }
  }
}

impl<T, Init, Folder, Repeater: Repeat> ops::Mul<Repeater> for InlineFold<T, Init, Folder> {
  type Output = Combinator<Mul<InlineFold<T, Init, Folder>, Repeater>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  fn mul(self, rhs: Repeater) -> Self::Output {
    Self::Output::new(Mul::new(self, rhs))
  }
}

unsafe impl<
    State,
    Heap,
    T: Action<State, Heap>,
    Acc,
    Repeater: Repeat,
    Init: Fn() -> Acc,
    Folder: Fn(T::Value, Acc, Input<&mut State, &mut Heap>) -> Acc,
  > Action<State, Heap> for Mul<InlineFold<T, Init, Folder>, Repeater>
{
  type Value = Acc;

  #[inline]
  fn exec(&self, mut input: Input<&mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    let repeat = &self.rhs;
    impl_mul!(input, repeat, self.lhs.init, self.lhs.fold, self.lhs.action)
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
    let combinator = eat('a').bind(1).fold(|| 0, |v, acc, _| acc + v) * (1..);
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
