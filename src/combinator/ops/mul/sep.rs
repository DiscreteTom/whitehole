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
