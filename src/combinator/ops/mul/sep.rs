use super::{inline::InlineFold, Fold, Mul, Repeat};
use crate::{
  action::{shift_input, Action, Input, Output},
  combinator::Combinator,
};

/// See [`Combinator::sep`].
#[derive(Debug, Clone, Copy)]
pub struct Sep<T, S> {
  value: T,
  sep: S,
}

impl<Lhs, Rhs> Combinator<Mul<Lhs, Rhs>> {
  /// Specify an other combinator as the separator
  /// after performing `*` on [`Combinator`]s.
  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  /// # Examples
  /// ```
  /// # use whitehole::{combinator::{eat, Combinator}, action::Action};
  /// # fn t(_: Combinator<impl Action>) {}
  /// # t(
  /// (eat("true") * (1..)).sep(eat(',')) // with a combinator
  /// # );
  /// ```
  /// You can use [`char`], `&str`, [`String`], and [`usize`] as the shorthand
  /// for [`eat`](crate::combinator::eat) in the separator.
  /// ```
  /// # use whitehole::{combinator::{eat, Combinator}, action::Action};
  /// # fn t(_: Combinator<impl Action>) {}
  /// # t(
  /// (eat("true") * (1..)).sep(',') // with a char
  /// # );
  /// # t(
  /// (eat("true") * (1..)).sep(",") // with a str
  /// # );
  /// # t(
  /// (eat("true") * (1..)).sep(",".to_string()) // with a string
  /// # );
  /// # t(
  /// (eat("true") * (1..)).sep(1) // with a usize
  /// # );
  /// ```
  #[inline]
  pub fn sep<S>(self, sep: impl Into<Combinator<S>>) -> Combinator<Sep<Mul<Lhs, Rhs>, S>> {
    Combinator::new(Sep {
      value: self.action,
      sep: sep.into().action,
    })
  }
}

macro_rules! impl_mul_with_sep {
  ($input:ident, $repeat:expr, $init:expr, $fold:expr, $action:expr, $sep:expr) => {{
    let mut repeated = 0;
    let mut output = Output {
      value: $init(),
      digested: 0,
    };

    let mut digested_with_sep = 0;
    while unsafe { $repeat.validate(repeated) } {
      let Some(value_output) = $action.exec(shift_input!($input, digested_with_sep)) else {
        break;
      };
      repeated += 1;
      output.value = $fold(value_output.value, output.value, $input.reborrow());
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - digested_with_sep > value_output.digested);
      output.digested = unsafe { digested_with_sep.unchecked_add(value_output.digested) };

      let Some(sep_output) = $sep.exec(shift_input!($input, output.digested)) else {
        break;
      };
      // SAFETY: since `slice::len` is usize, so `output.digested` must be a valid usize
      debug_assert!(usize::MAX - output.digested > sep_output.digested);
      digested_with_sep = unsafe { output.digested.unchecked_add(sep_output.digested) };
    }

    $repeat.accept(repeated).then_some(output)
  }};
}

unsafe impl<
    Lhs: Action<State, Heap, Value: Fold<State, Heap>>,
    Rhs: Repeat,
    S: Action<State, Heap>,
    State,
    Heap,
  > Action<State, Heap> for Sep<Mul<Combinator<Lhs>, Rhs>, S>
{
  type Value = <Lhs::Value as Fold<State, Heap>>::Output;

  #[inline]
  fn exec(&self, mut input: Input<&mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    impl_mul_with_sep!(
      input,
      self.value.rhs,
      Default::default,
      Fold::fold,
      self.value.lhs,
      self.sep
    )
  }
}

unsafe impl<
    T: Action<State, Heap>,
    Acc,
    Repeater: Repeat,
    Init: Fn() -> Acc,
    Folder: Fn(T::Value, Acc, Input<&mut State, &mut Heap>) -> Acc,
    S: Action<State, Heap>,
    State,
    Heap,
  > Action<State, Heap> for Sep<Mul<InlineFold<T, Init, Folder>, Repeater>, S>
{
  type Value = Acc;

  #[inline]
  fn exec(&self, mut input: Input<&mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    impl_mul_with_sep!(
      input,
      self.value.rhs,
      self.value.lhs.init,
      self.value.lhs.fold,
      self.value.lhs.action,
      self.sep
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::eat, instant::Instant};

  #[test]
  fn combinator_mul_with_sep() {
    let one_or_more = || (eat('a') * (1..)).sep(',');
    macro_rules! input {
      ($rest:expr) => {
        Input::new(Instant::new($rest), &mut (), &mut ())
      };
    }

    assert_eq!(one_or_more().exec(input!(",")), None);
    assert_eq!(
      one_or_more().exec(input!("a")),
      Some(Output {
        value: (),
        digested: 1
      })
    );
    assert_eq!(
      one_or_more().exec(input!("a,")),
      Some(Output {
        value: (),
        digested: 1
      })
    );
    assert_eq!(
      one_or_more().exec(input!("a,a")),
      Some(Output {
        value: (),
        digested: 3
      })
    );
    assert_eq!(
      one_or_more().exec(input!("a,,")),
      Some(Output {
        value: (),
        digested: 1
      })
    );
    assert_eq!(
      one_or_more().exec(input!("a,aa")),
      Some(Output {
        value: (),
        digested: 3
      })
    );
  }

  #[test]
  fn test_inline_fold_with_sep() {
    let combinator = (eat('a').bind(1).fold(|| 0, |v, acc, _| acc + v) * (1..)).sep(',');
    let output = combinator
      .exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ()))
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 5);
  }
}
