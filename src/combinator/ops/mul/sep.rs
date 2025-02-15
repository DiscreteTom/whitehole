use super::Mul;
use crate::{
  action::{Action, Context, Output},
  combinator::Combinator,
  instant::Instant,
};

/// A util struct to represent no separator.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoSep;

unsafe impl<Text: ?Sized, State, Heap> Action<Text, State, Heap> for NoSep {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    _: &Instant<&Text>,
    _: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
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
  /// # use whitehole::{combinator::{eat, Combinator}, action::{Action, Context}, instant::Instant};
  /// // eat `true` for 1 or more times, separated by `,` with optional spaces
  /// let action = {
  ///   let ws = || eat(' ') * (..);
  ///   (eat("true") * (1..)).sep(ws() + eat(',') + ws())
  /// };
  /// assert!(action.exec(&Instant::new("true"), Context::default()).is_some());
  /// assert!(action.exec(&Instant::new("true,true"), Context::default()).is_some());
  /// assert!(action.exec(&Instant::new("true , true"), Context::default()).is_some());
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
  /// # use whitehole::{combinator::eat, action::{Action, Context}, instant::Instant};
  /// let combinator = (eat('a').bind(1) * (1..)).sep(',').fold(|| 0, |v, acc| acc + v);
  /// assert_eq!(
  ///   combinator.exec(&Instant::new("a,a,a"), Context::default()).unwrap().value,
  ///   3
  /// );
  /// let combinator = (eat('a').bind(1) * (1..)).fold(|| 0, |v, acc| acc + v).sep(',');
  /// assert_eq!(
  ///   combinator.exec(&Instant::new("a,a,a"), Context::default()).unwrap().value,
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

    assert_eq!(
      one_or_more().exec(&Instant::new(","), Context::default()),
      None
    );
    assert_eq!(
      one_or_more().exec(&Instant::new("a"), Context::default()),
      Some(Output {
        value: (),
        digested: 1
      })
    );
    assert_eq!(
      one_or_more().exec(&Instant::new("a,"), Context::default()),
      Some(Output {
        value: (),
        digested: 1
      })
    );
    assert_eq!(
      one_or_more().exec(&Instant::new("a,a"), Context::default()),
      Some(Output {
        value: (),
        digested: 3
      })
    );
    assert_eq!(
      one_or_more().exec(&Instant::new("a,,"), Context::default()),
      Some(Output {
        value: (),
        digested: 1
      })
    );
    assert_eq!(
      one_or_more().exec(&Instant::new("a,aa"), Context::default()),
      Some(Output {
        value: (),
        digested: 3
      })
    );
  }

  #[test]
  fn test_fold_with_sep() {
    let combinator = (eat('a').bind(1) * (1..))
      .fold(|| 0, |acc, v| acc + v)
      .sep(',');
    let output = combinator
      .exec(&Instant::new("a,a,a"), Context::default())
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 5);
  }

  #[test]
  fn test_sep_with_eat() {
    fn t(action: Combinator<impl Action>) {
      assert!(action
        .exec(&Instant::new("true"), Context::default())
        .is_some());
      assert!(action
        .exec(&Instant::new("true,true"), Context::default())
        .is_some());
    }
    fn tb(action: Combinator<impl Action<[u8]>>) {
      assert!(action
        .exec(&Instant::new(b"true"), Context::default())
        .is_some());
      assert!(action
        .exec(&Instant::new(b"true,true"), Context::default())
        .is_some());
    }
    // with a char
    t((eat("true") * (1..)).sep(','));
    // with a str
    t((eat("true") * (1..)).sep(","));
    // with a string
    t((eat("true") * (1..)).sep(",".to_string()));
    // with a u8
    tb((eat(b"true") * (1..)).sep(b','));
    // with a &[u8]
    tb((eat(b"true") * (1..)).sep(b","));
    // with a Vec<u8>
    tb((eat(b"true") * (1..)).sep(vec![b',']));
  }
}
