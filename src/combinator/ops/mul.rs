//! Overload `*` operator for [`Combinator`].

mod fold;
mod repeat;

pub use fold::*;
pub use repeat::*;

use crate::combinator::{Action, Combinator, EatChar, EatStr, EatString, Input, Output};
use std::ops;

/// An [`Action`] created by `*`.
#[derive(Debug, Clone, Copy)]
pub struct Mul<Lhs, Rhs> {
  pub lhs: Lhs,
  pub rhs: Rhs,
}

impl<Lhs, Rhs> Mul<Lhs, Rhs> {
  #[inline]
  pub const fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self { lhs, rhs }
  }
}

impl<
    Lhs: Action,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc) -> Acc,
  > ops::Mul<(Repeater, Initializer, InlineFolder)> for Combinator<Lhs>
{
  type Output = Combinator<Mul<Lhs, (Repeater, Initializer, InlineFolder)>>;

  /// Create a new combinator to repeat the original combinator
  /// with the given repetition range, accumulator initializer and folder.
  /// The combinator will return the output with the [`Fold`]-ed value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: (Repeater, Initializer, InlineFolder)) -> Self::Output {
    Self::Output::new(Mul::new(self.action, rhs))
  }
}

#[inline]
fn impl_mul<'text, Value, State, Heap, Acc>(
  lhs: &impl Action<Value = Value, State = State, Heap = Heap>,
  range: &impl Repeat,
  init: impl Fn() -> Acc,
  folder: impl Fn(Value, Acc) -> Acc,
  input: &mut Input<'text, &mut State, &mut Heap>,
) -> Option<Output<'text, Acc>> {
  let mut repeated = 0;
  let mut output = Output {
    value: init(),
    rest: input.rest(),
  };

  while range.validate(repeated) {
    let Some(next_output) = input
      .reload(output.rest)
      .and_then(|mut input| lhs.exec(&mut input))
    else {
      break;
    };
    output.rest = next_output.rest;
    output.value = folder(next_output.value, output.value);
    repeated += 1;
  }

  range.accept(repeated).then_some(output)
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

  /// Create a new combinator to repeat the original combinator
  /// with the given repetition range, separator, accumulator initializer and folder.
  /// The combinator will return the output with the [`Fold`]-ed value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// If there is at least one repetition, then the separator is allowed to be the last match.
  /// E.g. `eat('a') * (1.., eat(','))` will accept `"a"`, `"a,"`, `"a,a"` but reject `","`.
  ///
  /// See [`Fold`] for more information.
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

  /// Create a new combinator to repeat the original combinator
  /// with the given repetition range, separator, accumulator initializer and folder.
  /// The combinator will return the output with the [`Fold`]-ed value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// If there is at least one repetition, then the separator is allowed to be the last match.
  /// E.g. `eat('a') * (1.., eat(','))` will accept `"a"`, `"a,"`, `"a,a"` but reject `","`.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: (Repeater, char, Initializer, InlineFolder)) -> Self::Output {
    let (range, sep, init, folder) = rhs;
    Self::Output::new(Mul::new(
      self.action,
      (range, EatChar::new(sep), init, folder),
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

  /// Create a new combinator to repeat the original combinator
  /// with the given repetition range, separator, accumulator initializer and folder.
  /// The combinator will return the output with the [`Fold`]-ed value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// If there is at least one repetition, then the separator is allowed to be the last match.
  /// E.g. `eat('a') * (1.., eat(','))` will accept `"a"`, `"a,"`, `"a,a"` but reject `","`.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: (Repeater, String, Initializer, InlineFolder)) -> Self::Output {
    let (range, sep, init, folder) = rhs;
    Self::Output::new(Mul::new(
      self.action,
      (range, EatString::new(sep), init, folder),
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

  /// Create a new combinator to repeat the original combinator
  /// with the given repetition range, separator, accumulator initializer and folder.
  /// The combinator will return the output with the [`Fold`]-ed value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// If there is at least one repetition, then the separator is allowed to be the last match.
  /// E.g. `eat('a') * (1.., eat(','))` will accept `"a"`, `"a,"`, `"a,a"` but reject `","`.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: (Repeater, &'a str, Initializer, InlineFolder)) -> Self::Output {
    let (range, sep, init, folder) = rhs;
    Self::Output::new(Mul::new(
      self.action,
      (range, EatStr::new(sep), init, folder),
    ))
  }
}

#[inline]
fn impl_mul_with_sep<'text, Value, State, Heap, Acc>(
  lhs: &impl Action<Value = Value, State = State, Heap = Heap>,
  range: &impl Repeat,
  sep: &impl Action<Value = (), State = State, Heap = Heap>,
  init: impl Fn() -> Acc,
  folder: impl Fn(Value, Acc) -> Acc,
  input: &mut Input<'text, &mut State, &mut Heap>,
) -> Option<Output<'text, Acc>> {
  let mut repeated = 0;
  let mut output = Output {
    value: init(),
    rest: input.rest(),
  };

  while range.validate(repeated) {
    let Some(next_output) = input
      .reload(output.rest)
      .and_then(|mut input| lhs.exec(&mut input))
    else {
      break;
    };
    repeated += 1;
    output.rest = next_output.rest;
    output.value = folder(next_output.value, output.value);
    let Some(next_output) = input
      .reload(next_output.rest)
      .and_then(|mut input| sep.exec(&mut input))
    else {
      break;
    };
    output.rest = next_output.rest;
  }

  range.accept(repeated).then_some(output)
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

impl<Lhs: Action<Value: Fold>, Rhs: Repeat> ops::Mul<Rhs> for Combinator<Lhs> {
  type Output = Combinator<Mul<Lhs, Rhs>>;

  /// Create a new combinator to repeat the original combinator for `rhs` times.
  /// The combinator will return the output with the [`Fold`]-ed value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul::new(self.action, rhs))
  }
}

impl<Lhs: Action<Value: Fold>, Rhs: Repeat> Action for Mul<Lhs, Rhs> {
  type Value = <Lhs::Value as Fold>::Output;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Value>> {
    impl_mul(
      &self.lhs,
      &self.rhs,
      Self::Value::default,
      Lhs::Value::fold,
      input,
    )
  }
}

impl<
    Lhs: Action<Value: Fold>,
    Repeater: Repeat,
    Sep: Action<Value = (), State = Lhs::State, Heap = Lhs::Heap>,
  > ops::Mul<(Repeater, Combinator<Sep>)> for Combinator<Lhs>
{
  type Output = Combinator<Mul<Lhs, (Repeater, Sep)>>;

  /// Create a new combinator to repeat the original combinator
  /// with the given repetition range, separator, accumulator initializer and folder.
  /// The combinator will return the output with the [`Fold`]-ed value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// If there is at least one repetition, then the separator is allowed to be the last match.
  /// E.g. `eat('a') * (1.., eat(','))` will accept `"a"`, `"a,"`, `"a,a"` but reject `","`.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: (Repeater, Combinator<Sep>)) -> Self::Output {
    let (range, sep) = rhs;
    Self::Output::new(Mul::new(self.action, (range, sep.action)))
  }
}

impl<Lhs: Action<Value: Fold>, Repeater: Repeat> ops::Mul<(Repeater, char)> for Combinator<Lhs> {
  type Output = Combinator<Mul<Lhs, (Repeater, EatChar<Lhs::State, Lhs::Heap>)>>;

  /// Create a new combinator to repeat the original combinator
  /// with the given repetition range, separator, accumulator initializer and folder.
  /// The combinator will return the output with the [`Fold`]-ed value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// If there is at least one repetition, then the separator is allowed to be the last match.
  /// E.g. `eat('a') * (1.., eat(','))` will accept `"a"`, `"a,"`, `"a,a"` but reject `","`.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: (Repeater, char)) -> Self::Output {
    let (range, sep) = rhs;
    Self::Output::new(Mul::new(self.action, (range, EatChar::new(sep))))
  }
}

impl<Lhs: Action<Value: Fold>, Repeater: Repeat> ops::Mul<(Repeater, String)> for Combinator<Lhs> {
  type Output = Combinator<Mul<Lhs, (Repeater, EatString<Lhs::State, Lhs::Heap>)>>;

  /// Create a new combinator to repeat the original combinator
  /// with the given repetition range, separator, accumulator initializer and folder.
  /// The combinator will return the output with the [`Fold`]-ed value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// If there is at least one repetition, then the separator is allowed to be the last match.
  /// E.g. `eat('a') * (1.., eat(','))` will accept `"a"`, `"a,"`, `"a,a"` but reject `","`.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: (Repeater, String)) -> Self::Output {
    let (range, sep) = rhs;
    Self::Output::new(Mul::new(self.action, (range, EatString::new(sep))))
  }
}

impl<'a, Lhs: Action<Value: Fold>, Repeater: Repeat> ops::Mul<(Repeater, &'a str)>
  for Combinator<Lhs>
{
  type Output = Combinator<Mul<Lhs, (Repeater, EatStr<'a, Lhs::State, Lhs::Heap>)>>;

  /// Create a new combinator to repeat the original combinator
  /// with the given repetition range, separator, accumulator initializer and folder.
  /// The combinator will return the output with the [`Fold`]-ed value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// If there is at least one repetition, then the separator is allowed to be the last match.
  /// E.g. `eat('a') * (1.., eat(','))` will accept `"a"`, `"a,"`, `"a,a"` but reject `","`.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: (Repeater, &'a str)) -> Self::Output {
    let (range, sep) = rhs;
    Self::Output::new(Mul::new(self.action, (range, EatStr::new(sep))))
  }
}

impl<
    Lhs: Action<Value: Fold>,
    Repeater: Repeat,
    Sep: Action<Value = (), State = Lhs::State, Heap = Lhs::Heap>,
  > Action for Mul<Lhs, (Repeater, Sep)>
{
  type Value = <Lhs::Value as Fold>::Output;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut Lhs::State, &mut Lhs::Heap>,
  ) -> Option<Output<'text, Self::Value>> {
    let (range, sep) = &self.rhs;
    impl_mul_with_sep(
      &self.lhs,
      range,
      sep,
      Self::Value::default,
      Lhs::Value::fold,
      input,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::{wrap, Input, Output};

  #[derive(Debug)]
  struct MyValue(usize);
  impl Fold for MyValue {
    type Output = usize;
    fn fold(self, current: Self::Output) -> Self::Output {
      self.0 + current
    }
  }

  #[test]
  fn combinator_mul_usize() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * 3)
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    let n = 0;
    assert_eq!(
      (rejecter() * n).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    let n = 0;
    assert_eq!(
      (accepter() * n).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * 3).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );

    // overflow, reject
    assert!((accepter() * 4)
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..2))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..2)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..1)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 1,
        rest: "3"
      })
    );

    // too few, reject
    assert!((accepter() * (4..6))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_from() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );

    // too few, reject
    assert!((accepter() * (4..))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_full() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );
  }

  #[test]
  fn combinator_mul_range_inclusive() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * (1..=3))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..=2)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..=0)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..=3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );

    // too few, reject
    assert!((accepter() * (4..=6))
      .exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }

  #[test]
  fn combinator_mul_range_to() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..2)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..1)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 1,
        rest: "3"
      })
    );
  }

  #[test]
  fn combinator_mul_range_to_inclusive() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter = || {
      wrap(|input| {
        Some(Output {
          value: MyValue(input.start()),
          rest: &input.rest()[1..],
        })
      })
    };

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (..=2)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..=0)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..=3)).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );
  }

  #[test]
  fn combinator_mul_with_sep() {
    let eat_char = |c| {
      wrap(move |input| {
        (input.next() == c).then(|| Output {
          value: (),
          rest: &input.rest()[1..],
        })
      })
    };
    let eat_a = || eat_char('a');
    let sep = || eat_char(',');

    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new(",", 0, &mut (), &mut ()).unwrap()),
      None
    );
    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new("a", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ""
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new("a,", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ""
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new("a,a", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ""
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new("a,,", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ","
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).exec(&mut Input::new("a,aa", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "a"
      })
    );

    // TODO: more tests
  }
}
