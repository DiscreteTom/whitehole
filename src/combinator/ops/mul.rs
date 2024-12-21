//! Overload `*` operator for [`Combinator`].

use crate::combinator::{Combinator, EatChar, EatStr, EatString, Input, Output, Parse};
use std::ops::{self, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/// A helper trait to represent repetition when performing `*` on [`Combinator`]s.
///
/// Built-in implementations are provided for
/// [`usize`], [`Range<usize>`], [`RangeFrom<usize>`], [`RangeFull`],
/// [`RangeInclusive<usize>`], [`RangeTo<usize>`], and [`RangeToInclusive<usize>`].
pub trait Repeat {
  /// Check if the repetition should continue
  /// based on the current repeated times.
  fn validate(&self, repeated: usize) -> bool;

  /// Check if the repetition should be accepted
  /// based on the current repeated times.
  fn accept(&self, repeated: usize) -> bool;
}

impl Repeat for usize {
  #[inline]
  fn validate(&self, repeated: usize) -> bool {
    repeated < *self
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    repeated == *self
  }
}

impl Repeat for Range<usize> {
  #[inline]
  fn validate(&self, repeated: usize) -> bool {
    repeated + 1 < self.end
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeFrom<usize> {
  #[inline]
  fn validate(&self, _: usize) -> bool {
    true
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeFull {
  #[inline]
  fn validate(&self, _: usize) -> bool {
    true
  }

  #[inline]
  fn accept(&self, _: usize) -> bool {
    true
  }
}

impl Repeat for RangeInclusive<usize> {
  #[inline]
  fn validate(&self, repeated: usize) -> bool {
    repeated < *self.end()
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeTo<usize> {
  #[inline]
  fn validate(&self, repeated: usize) -> bool {
    repeated + 1 < self.end
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

impl Repeat for RangeToInclusive<usize> {
  #[inline]
  fn validate(&self, repeated: usize) -> bool {
    repeated < self.end
  }

  #[inline]
  fn accept(&self, repeated: usize) -> bool {
    self.contains(&repeated)
  }
}

/// A [`Parse`] implementor created by `*`.
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
    Lhs: Parse,
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
    Self::Output::new(Mul::new(self.parser, rhs))
  }
}

#[inline]
fn impl_mul<'text, Value, State, Heap, Acc>(
  lhs: &impl Parse<Value = Value, State = State, Heap = Heap>,
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
      .and_then(|mut input| lhs.parse(&mut input))
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
    Lhs: Parse,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc) -> Acc,
  > Parse for Mul<Lhs, (Repeater, Initializer, InlineFolder)>
{
  type Value = Acc;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Acc>> {
    let (range, init, folder) = &self.rhs;
    impl_mul(&self.lhs, range, init, folder, input)
  }
}

impl<
    Lhs: Parse,
    Sep: Parse<Value = (), State = Lhs::State, Heap = Lhs::Heap>, // TODO: allow more generic Value
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
    Self::Output::new(Mul::new(self.parser, (range, sep.parser, init, folder)))
  }
}

impl<
    Lhs: Parse,
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
      self.parser,
      (range, EatChar::new(sep), init, folder),
    ))
  }
}

impl<
    Lhs: Parse,
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
      self.parser,
      (range, EatString::new(sep), init, folder),
    ))
  }
}

impl<
    'a,
    Lhs: Parse,
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
      self.parser,
      (range, EatStr::new(sep), init, folder),
    ))
  }
}

#[inline]
fn impl_mul_with_sep<'text, Value, State, Heap, Acc>(
  lhs: &impl Parse<Value = Value, State = State, Heap = Heap>,
  range: &impl Repeat,
  sep: &impl Parse<Value = (), State = State, Heap = Heap>,
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
      .and_then(|mut input| lhs.parse(&mut input))
    else {
      break;
    };
    repeated += 1;
    output.rest = next_output.rest;
    output.value = folder(next_output.value, output.value);
    let Some(next_output) = input
      .reload(next_output.rest)
      .and_then(|mut input| sep.parse(&mut input))
    else {
      break;
    };
    output.rest = next_output.rest;
  }

  range.accept(repeated).then_some(output)
}

impl<
    Lhs: Parse,
    Sep: Parse<Value = (), State = Lhs::State, Heap = Lhs::Heap>,
    Acc,
    Repeater: Repeat,
    Initializer: Fn() -> Acc,
    InlineFolder: Fn(Lhs::Value, Acc) -> Acc,
  > Parse for Mul<Lhs, (Repeater, Sep, Initializer, InlineFolder)>
{
  type Value = Acc;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Acc>> {
    let (range, sep, init, folder) = &self.rhs;
    impl_mul_with_sep(&self.lhs, range, sep, init, folder, input)
  }
}

/// A helper trait to accumulate values when performing `*` on [`Combinator`]s.
///
/// Built-in implementations are provided for `()`.
/// # Examples
/// ## Inline Fold
/// For simple cases, you can accumulate the values inline, without using this trait.
/// ```
/// # use whitehole::{combinator::next, parse::{Input, Parse}};
/// let combinator =
///   // accept one ascii digit at a time
///   next(|c| c.is_ascii_digit())
///     // convert the char to a number
///     .select(|ctx| ctx.input.next() as usize - '0' as usize)
///     // repeat for 1 or more times, init accumulator with 0, and fold values
///     * (1.., || 0 as usize, |value, acc| acc * 10 + value);
///
/// // parse "123" to 123
/// assert_eq!(
///   combinator.parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()).unwrap().value,
///   123
/// )
/// ```
/// ## Fold with Custom Type
/// If you want to re-use the folder logic, you can implement this trait for a custom type.
/// ```
/// # use whitehole::{combinator::{ops::mul::Fold, next}, parse::{Input, Parse}};
/// // since you can't implement `Fold` for `usize` directly,
/// // wrap it in a new-type
/// struct Usize(usize);
///
/// impl Fold for Usize {
///   type Output = usize;
///
///   fn fold(self, acc: Self::Output) -> Self::Output {
///     acc * 10 + self.0
///   }
/// }
///
/// let combinator =
///   // accept one ascii digit at a time
///   next(|c| c.is_ascii_digit())
///     // convert the char to a number, wrapped in `Usize`
///     .select(|ctx| Usize(ctx.input.next() as usize - '0' as usize))
///     // repeat for 1 or more times, fold `Usize` to `usize`
///     * (1..);
///     // equals to: `* (1.., Usize::Output::default, Usize::fold)`
///
/// // parse "123" to 123
/// assert_eq!(
///   combinator.parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()).unwrap().value,
///   123
/// )
/// ```
pub trait Fold {
  /// The accumulator type.
  type Output: Default;

  /// Fold self with the accumulator.
  fn fold(self, acc: Self::Output) -> Self::Output;
}

impl Fold for () {
  type Output = ();
  #[inline]
  fn fold(self, _: Self::Output) -> Self::Output {}
}

impl<Lhs: Parse<Value: Fold>, Rhs: Repeat> ops::Mul<Rhs> for Combinator<Lhs> {
  type Output = Combinator<Mul<Lhs, Rhs>>;

  /// Create a new combinator to repeat the original combinator for `rhs` times.
  /// The combinator will return the output with the [`Fold`]-ed value and the sum of the digested,
  /// or reject if the repetition is not satisfied.
  ///
  /// `0` is a valid repetition range, which means the combinator is optional.
  ///
  /// See [`Fold`] for more information.
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul::new(self.parser, rhs))
  }
}

impl<Lhs: Parse<Value: Fold>, Rhs: Repeat> Parse for Mul<Lhs, Rhs> {
  type Value = <Lhs::Value as Fold>::Output;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn parse<'text>(
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
    Lhs: Parse<Value: Fold>,
    Repeater: Repeat,
    Sep: Parse<Value = (), State = Lhs::State, Heap = Lhs::Heap>,
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
    Self::Output::new(Mul::new(self.parser, (range, sep.parser)))
  }
}

impl<Lhs: Parse<Value: Fold>, Repeater: Repeat> ops::Mul<(Repeater, char)> for Combinator<Lhs> {
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
    Self::Output::new(Mul::new(self.parser, (range, EatChar::new(sep))))
  }
}

impl<Lhs: Parse<Value: Fold>, Repeater: Repeat> ops::Mul<(Repeater, String)> for Combinator<Lhs> {
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
    Self::Output::new(Mul::new(self.parser, (range, EatString::new(sep))))
  }
}

impl<'a, Lhs: Parse<Value: Fold>, Repeater: Repeat> ops::Mul<(Repeater, &'a str)>
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
    Self::Output::new(Mul::new(self.parser, (range, EatStr::new(sep))))
  }
}

impl<
    Lhs: Parse<Value: Fold>,
    Repeater: Repeat,
    Sep: Parse<Value = (), State = Lhs::State, Heap = Lhs::Heap>,
  > Parse for Mul<Lhs, (Repeater, Sep)>
{
  type Value = <Lhs::Value as Fold>::Output;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn parse<'text>(
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
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    let n = 0;
    assert_eq!(
      (rejecter() * n).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    let n = 0;
    assert_eq!(
      (accepter() * n).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * 3).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );

    // overflow, reject
    assert!((accepter() * 4)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
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
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..1)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 1,
        rest: "3"
      })
    );

    // too few, reject
    assert!((accepter() * (4..6))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
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
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );

    // too few, reject
    assert!((accepter() * (4..))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
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
      (rejecter() * (..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
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
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat rejecter 0 times will accept
    assert_eq!(
      (rejecter() * (0..=2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (0..=0)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (0..=3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { value: 3, rest: "" })
    );

    // too few, reject
    assert!((accepter() * (4..=6))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
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
      (rejecter() * (..2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..1)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
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
      (rejecter() * (..=2)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123",
      })
    );

    // repeat an accepter 0 times will accept
    assert_eq!(
      (accepter() * (..=0)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        rest: "123",
      })
    );

    // normal, apply the folded value and sum the digested
    assert_eq!(
      (accepter() * (..=3)).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
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
      (eat_a() * (1.., sep())).parse(&mut Input::new(",", 0, &mut (), &mut ()).unwrap()),
      None
    );
    assert_eq!(
      (eat_a() * (1.., sep())).parse(&mut Input::new("a", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ""
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).parse(&mut Input::new("a,", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ""
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).parse(&mut Input::new("a,a", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ""
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).parse(&mut Input::new("a,,", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ","
      })
    );
    assert_eq!(
      (eat_a() * (1.., sep())).parse(&mut Input::new("a,aa", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "a"
      })
    );

    // TODO: more tests
  }
}
