use super::{impl_mul, Mul, Repeat};
use crate::{
  action::{Action, Input, Output},
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
  /// Specify accumulator initializer and folder in an inline way.
  ///
  /// The return value is not a [`Combinator`], you should use `*` to combine it with a [`Repeat`].
  ///
  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  /// # Examples
  /// ```
  /// # use whitehole::{combinator::next, action::{Input, Action}, instant::Instant};
  /// let combinator =
  ///   // accept one ascii digit at a time
  ///   next(|c| c.is_ascii_digit())
  ///     // convert the char to a number
  ///     .select(|ctx| ctx.input().instant().rest().chars().next().unwrap() as usize - '0' as usize)
  ///     // init accumulator with 0, and fold values
  ///     .fold(|| 0 as usize, |value, acc, _| acc * 10 + value)
  ///     // repeat for 1 or more times
  ///     * (1..);
  ///
  /// // parse "123" to 123
  /// assert_eq!(
  ///   combinator.exec(Input::new(Instant::new("123"), &mut (), &mut ())).unwrap().value,
  ///   123
  /// )
  /// ```
  #[inline]
  pub fn fold<
    State,
    Heap,
    Acc,
    Init: Fn() -> Acc,
    Folder: Fn(T::Value, Acc, Input<&str, &mut State, &mut Heap>) -> Acc,
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
    Folder: Fn(T::Value, Acc, Input<&str, &mut State, &mut Heap>) -> Acc,
  > Action<State, Heap> for Mul<InlineFold<T, Init, Folder>, Repeater>
{
  type Value = Acc;

  #[inline]
  fn exec(&self, mut input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
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
      .exec(Input::new(Instant::new("aaa"), &mut (), &mut ()))
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 3);
  }
}
