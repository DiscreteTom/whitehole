use super::{impl_mul, Mul, Repeat};
use crate::{
  action::{shift_input, Action, Input, Output},
  combinator::Combinator,
};
use core::fmt;
use std::ops;

/// See [`Combinator::fold`].
#[derive(Copy, Clone)]
pub struct InlineFold<T, Init, Folder> {
  pub(super) action: T,
  pub(super) init: Init,
  pub(super) fold: Folder,
}

impl<T: fmt::Debug, Init, Folder> fmt::Debug for InlineFold<T, Init, Folder> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let action = &self.action;
    f.debug_struct(stringify!(InlineFold))
      .field(stringify!(action), action)
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
    let combinator = (eat('a').bind(1).fold(|| 0, |v, acc, _| acc + v) * (1..)).sep(',');
    let output = combinator
      .exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ()).unwrap())
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 5);
  }
}
