use super::{Fold, Mul, Repeat};
use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
};
use std::ops;

/// See [`Combinator::sep`].
#[derive(Debug, Clone, Copy)]
pub struct Sep<T, S> {
  pub value: T,
  pub sep: S,
}

impl<T> Combinator<T> {
  /// Specify an other combinator as the separator
  /// before performing `*` on [`Combinator`](crate::combinator::Combinator)s.
  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  /// # Examples
  /// ```
  /// # use whitehole::{combinator::eat, C};
  /// # fn t(_: C!()) {}
  /// # t(
  /// eat("true").sep(eat(',')) * (1..) // with a combinator
  /// # );
  /// ```
  /// You can use [`char`], `&str`, [`String`], and [`usize`] as the shorthand
  /// for [`eat`](crate::combinator::eat) in the separator.
  /// ```
  /// # use whitehole::{combinator::eat, C};
  /// # fn t(_: C!()) {}
  /// # t(
  /// eat("true").sep(',') * (1..) // with a char
  /// # );
  /// # t(
  /// eat("true").sep(",") * (1..) // with a str
  /// # );
  /// # t(
  /// eat("true").sep(",".to_string()) * (1..) // with a string
  /// # );
  /// # t(
  /// eat("true").sep(1) * (1..) // with a usize
  /// # );
  /// ```
  #[inline]
  pub fn sep<S>(self, sep: impl Into<Combinator<S>>) -> Sep<T, S> {
    Sep {
      value: self.action,
      sep: sep.into().action,
    }
  }
}

impl<T: Action<Value: Fold>, S: Action<State = T::State, Heap = T::Heap>, Rhs: Repeat> ops::Mul<Rhs>
  for Sep<T, S>
{
  type Output = Combinator<Mul<Sep<T, S>, Rhs>>;

  /// See [`ops::mul`](crate::combinator::ops::mul) for more information.
  fn mul(self, rhs: Rhs) -> Self::Output {
    Self::Output::new(Mul::new(self, rhs))
  }
}

unsafe impl<T: Action<Value: Fold>, S: Action<State = T::State, Heap = T::Heap>, Rhs: Repeat> Action
  for Mul<Sep<T, S>, Rhs>
{
  type Value = <T::Value as Fold>::Output;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec<'text>(
    &self,
    mut input: Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    if !self.rhs.validate(0) {
      return self.rhs.accept(0).then(|| Output {
        value: Default::default(),
        digested: 0,
      });
    }

    // the first occurrence of `value`
    let (mut repeated, mut output) = if let Some(output) = self.lhs.value.exec(input.reborrow()) {
      (1, output.map(|value| value.fold(Default::default())))
    } else {
      return self.rhs.accept(0).then(|| Output {
        value: Default::default(),
        digested: 0,
      });
    };

    // the rest of the occurrences
    while self.rhs.validate(repeated) {
      let Some(sep_output) = input
        .reload(output.digested)
        .and_then(|input| self.lhs.sep.exec(input))
      else {
        break;
      };
      let Some(value_output) = input
        .reload(output.digested + sep_output.digested)
        .and_then(|input| self.lhs.value.exec(input))
      else {
        break;
      };

      // now we have both `value` and `sep`, update `output` and `repeated`
      repeated += 1;
      output.digested += sep_output.digested + value_output.digested;
      output.value = value_output.value.fold(output.value);
    }

    self.rhs.accept(repeated).then_some(output)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::eat;

  #[test]
  fn combinator_mul_with_sep() {
    let one_or_more = || eat('a').sep(',') * (1..);
    macro_rules! input {
      ($rest:expr) => {
        Input::new($rest, 0, &mut (), &mut ()).unwrap()
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
}
