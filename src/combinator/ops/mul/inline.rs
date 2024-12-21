use super::{impl_mul, impl_mul_with_sep, Mul, Repeat};
use crate::{
  action::{Action, Input, Output},
  combinator::{Combinator, EatChar, EatStr, EatString},
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
  ) -> Option<Output<'text, Acc>> {
    let (range, init, folder) = &self.rhs;
    impl_mul(&self.lhs, range, init, folder, input)
  }
}

impl<
    Lhs: Action,
    Sep: Action<Value = (), State = Lhs::State, Heap = Lhs::Heap>, // TODO: allow more generic Value
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc) -> Acc,
  > ops::Mul<(Repeater, Combinator<Sep>, Initializer, InlineFolder)> for Combinator<Lhs>
{
  type Output = Combinator<Mul<Lhs, (Repeater, Sep, Initializer, InlineFolder)>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: (Repeater, Combinator<Sep>, Initializer, InlineFolder)) -> Self::Output {
    let (range, sep, init, folder) = rhs;
    Self::Output::new(Mul::new(self.action, (range, sep.action, init, folder)))
  }
}

impl<
    Lhs: Action,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc) -> Acc,
  > ops::Mul<(Repeater, char, Initializer, InlineFolder)> for Combinator<Lhs>
{
  type Output = Combinator<
    Mul<
      Lhs,
      (
        Repeater,
        EatChar<Lhs::State, Lhs::Heap>,
        Initializer,
        InlineFolder,
      ),
    >,
  >;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: (Repeater, char, Initializer, InlineFolder)) -> Self::Output {
    let (range, sep, init, folder) = rhs;
    Self::Output::new(Mul::new(
      self.action,
      (range, EatChar::new(sep), init, folder),
    ))
  }
}

impl<
    'a,
    Lhs: Action,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc) -> Acc,
  > ops::Mul<(Repeater, &'a str, Initializer, InlineFolder)> for Combinator<Lhs>
{
  type Output = Combinator<
    Mul<
      Lhs,
      (
        Repeater,
        EatStr<'a, Lhs::State, Lhs::Heap>,
        Initializer,
        InlineFolder,
      ),
    >,
  >;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: (Repeater, &'a str, Initializer, InlineFolder)) -> Self::Output {
    let (range, sep, init, folder) = rhs;
    Self::Output::new(Mul::new(
      self.action,
      (range, EatStr::new(sep), init, folder),
    ))
  }
}

impl<
    Lhs: Action,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc) -> Acc,
  > ops::Mul<(Repeater, String, Initializer, InlineFolder)> for Combinator<Lhs>
{
  type Output = Combinator<
    Mul<
      Lhs,
      (
        Repeater,
        EatString<Lhs::State, Lhs::Heap>,
        Initializer,
        InlineFolder,
      ),
    >,
  >;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: (Repeater, String, Initializer, InlineFolder)) -> Self::Output {
    let (range, sep, init, folder) = rhs;
    Self::Output::new(Mul::new(
      self.action,
      (range, EatString::new(sep), init, folder),
    ))
  }
}

impl<
    Lhs: Action,
    Sep: Action<Value = (), State = Lhs::State, Heap = Lhs::Heap>,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc) -> Acc,
  > Action for Mul<Lhs, (Repeater, Sep, Initializer, InlineFolder)>
{
  type Value = Acc;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Acc>> {
    let (range, sep, init, folder) = &self.rhs;
    impl_mul_with_sep(&self.lhs, range, sep, init, folder, input)
  }
}
