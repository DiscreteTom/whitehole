use super::Mul;
use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
};

/// A util struct to represent no separator.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoSep;

unsafe impl<Text: ?Sized, State, Heap> Action<Text, State, Heap> for NoSep {
  type Value = ();

  #[inline]
  fn exec(&self, _: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    // just accept without digesting
    Some(Output {
      value: (),
      digested: 0,
    })
  }
}

impl<Lhs, Rhs, Sep, Init, Fold> Combinator<Mul<Lhs, Rhs, Sep, Init, Fold>> {
  /// Specify an other combinator as the separator
  /// after performing `*` on [`Combinator`]s.
  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  /// # Examples
  /// ```
  /// # use whitehole::{combinator::{eat, Combinator}, action::Action};
  /// // eat `true` for 1 or more times, separated by `,` with optional spaces
  /// let action = {
  ///   let ws = || eat(' ') * (..);
  ///   (eat("true") * (1..)).sep(ws() + eat(',') + ws())
  /// };
  /// assert!(action.exec(Input::new(Instant::new("true"), &mut (), &mut ())).is_some());
  /// assert!(action.exec(Input::new(Instant::new("true,true"), &mut (), &mut ())).is_some());
  /// assert!(action.exec(Input::new(Instant::new("true , true"), &mut (), &mut ())).is_some());
  /// ```
  /// Tips: you can use [`char`], `&str`, [`String`], [`u8`], `&[u8]` and [`Vec<u8>`] as the shorthand
  /// for [`eat`](crate::combinator::eat) in the separator.
  /// ```
  /// # use whitehole::{combinator::{eat, Combinator}, action::Action};
  /// # fn t(_: Combinator<impl Action>) {}
  /// # fn tb(_: Combinator<impl Action<[u8]>>) {}
  /// # t(
  /// (eat("true") * (1..)).sep(',') // with a char
  /// # );
  /// # t(
  /// (eat("true") * (1..)).sep(",") // with a str
  /// # );
  /// # t(
  /// (eat("true") * (1..)).sep(",".to_string()) // with a string
  /// # );
  /// # tb(
  /// (eat(b"true") * (1..)).sep(b',') // with a u8
  /// # );
  /// # tb(
  /// (eat(b"true") * (1..)).sep(b",") // with a &[u8]
  /// # );
  /// # tb(
  /// (eat(b"true") * (1..)).sep(vec![b',']) // with a Vec<u8>
  /// # );
  /// ```
  /// You can use [`Combinator::sep`] with [`Combinator::fold`] in any order after `*`,
  /// since they are actually builder methods for [`Combinator<Mul>`].
  /// ```
  /// # use whitehole::{combinator::{ops::mul::Fold, eat}, action::{Input, Action}, instant::Instant};
  /// let combinator = (eat('a').bind(1) * (1..)).sep(',').fold(|| 0, |v, acc| acc + v);
  /// assert_eq!(
  ///   combinator.exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ())).unwrap().value,
  ///   3
  /// );
  /// let combinator = (eat('a').bind(1) * (1..)).fold(|| 0, |v, acc| acc + v).sep(',');
  /// assert_eq!(
  ///   combinator.exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ())).unwrap().value,
  ///   3
  /// );
  /// ```
  /// You can't use [`Combinator::fold`] to accumulate values in the separator combinator.
  /// You can fold values of the separator combinator to the heap.
  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  #[inline]
  pub fn sep<NewSep>(
    self,
    sep: impl Into<Combinator<NewSep>>,
  ) -> Combinator<Mul<Lhs, Rhs, NewSep, Init, Fold>> {
    Combinator::new(Mul {
      lhs: self.action.lhs,
      rhs: self.action.rhs,
      sep: sep.into().action,
      init: self.action.init,
      fold: self.action.fold,
    })
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
    let combinator = (eat('a').bind(1) * (1..))
      .fold(|| 0, |acc, v| acc + v)
      .sep(',');
    let output = combinator
      .exec(Input::new(Instant::new("a,a,a"), &mut (), &mut ()))
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 5);
  }
}
