use std::marker::PhantomData;

use super::Mul;
use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
  instant::Instant,
};

/// A util struct to represent no separator.
/// See [`ops::mul`](crate::combinator::ops::mul) for more information.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoSep<Text: ?Sized, State, Heap> {
  _text: PhantomData<Text>,
  _phantom: PhantomData<(State, Heap)>,
}

impl<Text: ?Sized, State, Heap> NoSep<Text, State, Heap> {
  #[inline]
  pub const fn new() -> Self {
    Self {
      _text: PhantomData,
      _phantom: PhantomData,
    }
  }
}

unsafe impl<Text: ?Sized, State, Heap> Action for NoSep<Text, State, Heap> {
  type Text = Text;
  type State = State;
  type Heap = Heap;
  type Value = ();

  #[inline]
  fn exec(
    &self,
    _: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
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
  /// # use whitehole::{combinator::eat, parser::Parser};
  /// // eat `true` for 1 or more times, separated by `,` with optional spaces
  /// let entry = {
  ///   let ws = || eat(' ') * (..);
  ///   (eat("true") * (1..)).sep(ws() + eat(',') + ws())
  /// };
  /// assert_eq!(Parser::builder().entry(&entry).build("true").next().unwrap().digested, 4);
  /// assert_eq!(Parser::builder().entry(&entry).build("true,true").next().unwrap().digested, 9);
  /// assert_eq!(Parser::builder().entry(&entry).build("true , true").next().unwrap().digested, 11);
  /// ```
  /// Tips: you can use [`char`], `&str`, [`String`], [`u8`], `&[u8]` and [`Vec<u8>`] as the shorthand
  /// for [`eat`](crate::combinator::eat) in the separator.
  /// ```
  /// # use whitehole::{combinator::{eat, Combinator}, action::Action};
  /// # fn t(_: Combinator<impl Action>) {}
  /// # fn tb(_: Combinator<impl Action<Text=[u8]>>) {}
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
  /// # use whitehole::{combinator::eat, parser::Parser};
  /// let entry = (eat('a').bind(1) * (1..)).sep(',').fold(|| 0, |v, acc| acc + v);
  /// assert_eq!(
  ///   Parser::builder().entry(entry).build("a,a,a").next().unwrap().value,
  ///   3
  /// );
  /// let entry = (eat('a').bind(1) * (1..)).fold(|| 0, |v, acc| acc + v).sep(',');
  /// assert_eq!(
  ///   Parser::builder().entry(entry).build("a,a,a").next().unwrap().value,
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
  use crate::{
    combinator::{bytes, eat},
    digest::Digest,
    instant::Instant,
  };
  use std::{ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text = Text, State = (), Heap = (), Value = ()>,
    input: &Text,
    digested: usize,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action
        .exec(Input {
          instant: &Instant::new(input),
          state: &mut (),
          heap: &mut ()
        })
        .map_or(0, |output| output.digested),
      digested
    )
  }

  #[test]
  fn combinator_mul_with_sep() {
    let one_or_more = || (eat('a') * (1..)).sep(',');

    helper(one_or_more(), ",", 0);
    helper(one_or_more(), "a", 1);
    helper(one_or_more(), "a,", 1);
    helper(one_or_more(), "a,a", 3);
    helper(one_or_more(), "a,,", 1);
    helper(one_or_more(), "a,aa", 3);
  }

  #[test]
  fn test_fold_with_sep() {
    let combinator = (eat('a').bind(1) * (1..))
      .fold(|| 0, |acc, v| acc + v)
      .sep(',');
    let output = combinator
      .exec(Input {
        instant: &Instant::new("a,a,a"),
        state: &mut (),
        heap: &mut (),
      })
      .unwrap();
    assert_eq!(output.value, 3);
    assert_eq!(output.digested, 5);
  }

  #[test]
  fn test_sep_with_eat() {
    fn t(action: Combinator<impl Action<Text = str, State = (), Heap = ()>>) {
      assert!(action
        .exec(Input {
          instant: &Instant::new("true"),
          state: &mut (),
          heap: &mut ()
        })
        .is_some());
      assert!(action
        .exec(Input {
          instant: &Instant::new("true,true"),
          state: &mut (),
          heap: &mut ()
        })
        .is_some());
    }
    fn tb(action: Combinator<impl Action<Text = [u8], State = (), Heap = ()>>) {
      assert!(action
        .exec(Input {
          instant: &Instant::new(b"true"),
          state: &mut (),
          heap: &mut ()
        })
        .is_some());
      assert!(action
        .exec(Input {
          instant: &Instant::new(b"true,true"),
          state: &mut (),
          heap: &mut ()
        })
        .is_some());
    }
    // with a char
    t((eat("true") * (1..)).sep(','));
    // with a str
    t((eat("true") * (1..)).sep(","));
    // with a string
    t((eat("true") * (1..)).sep(",".to_string()));
    // with a u8
    tb((bytes::eat(b"true") * (1..)).sep(b','));
    // with a &[u8]
    tb((bytes::eat(b"true") * (1..)).sep(b","));
    // with a Vec<u8>
    tb((bytes::eat(b"true") * (1..)).sep(vec![b',']));
  }
}
