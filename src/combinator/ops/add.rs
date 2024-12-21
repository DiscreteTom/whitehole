//! Overload `+` operator for [`Combinator`].

use crate::combinator::{Combinator, EatChar, EatStr, EatString, EatUsize, Input, Output, Parse};
use std::ops;

/// A helper trait to concat types when performing `+` on [`Combinator`]s.
///
/// Built-in implementations:
/// - `concat(T, ()) -> T`
/// - `concat((), (T1, T2, ...)) -> (T1, T2, ...)` for results with up to 12 elements.
/// - `concat((T1, T2, ...), (U1, U2, ...)) -> (T1, T2, ..., U1, U2, ...)`
///   for results with up to 12 elements.
pub trait Concat<Rhs> {
  /// The concat result.
  type Output;
  /// Concat self with the `rhs`.
  fn concat(self, rhs: Rhs) -> Self::Output;
}

impl<T> Concat<()> for T {
  type Output = T;
  #[inline]
  fn concat(self, _: ()) -> Self::Output {
    self
  }
}

macro_rules! impl_concat_for_unit {
  ($($rhs:ident),*) => {
    impl<$($rhs),*> Concat<($($rhs),*,)> for () {
      type Output = ($($rhs),*,);
      #[inline]
      fn concat(self, rhs: ($($rhs),*,)) -> Self::Output {
        rhs
      }
    }
  };
}
impl_concat_for_unit!(_1);
impl_concat_for_unit!(_1, _2);
impl_concat_for_unit!(_1, _2, _3);
impl_concat_for_unit!(_1, _2, _3, _4);
impl_concat_for_unit!(_1, _2, _3, _4, _5);
impl_concat_for_unit!(_1, _2, _3, _4, _5, _6);
impl_concat_for_unit!(_1, _2, _3, _4, _5, _6, _7);
impl_concat_for_unit!(_1, _2, _3, _4, _5, _6, _7, _8);
impl_concat_for_unit!(_1, _2, _3, _4, _5, _6, _7, _8, _9);
impl_concat_for_unit!(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10);
impl_concat_for_unit!(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11);
impl_concat_for_unit!(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12);

macro_rules! impl_concat_tuple {
  (($($lhs:ident),*), ($($rhs:ident),*)) => {
    impl<$($lhs),*,$($rhs),*> Concat<($($rhs),*,)> for ($($lhs),*,) {
      type Output = ($($lhs),*,$($rhs),*);
      #[inline]
      fn concat(self, rhs: ($($rhs),*,)) -> Self::Output {
        let ($($lhs),*,) = self;
        let ($($rhs),*,) = rhs;
        ($($lhs),*, $($rhs),*)
      }
    }
  };
}
impl_concat_tuple!((_1), (_2));
impl_concat_tuple!((_1), (_2, _3));
impl_concat_tuple!((_1), (_2, _3, _4));
impl_concat_tuple!((_1), (_2, _3, _4, _5));
impl_concat_tuple!((_1), (_2, _3, _4, _5, _6));
impl_concat_tuple!((_1), (_2, _3, _4, _5, _6, _7));
impl_concat_tuple!((_1), (_2, _3, _4, _5, _6, _7, _8));
impl_concat_tuple!((_1), (_2, _3, _4, _5, _6, _7, _8, _9));
impl_concat_tuple!((_1), (_2, _3, _4, _5, _6, _7, _8, _9, _10));
impl_concat_tuple!((_1), (_2, _3, _4, _5, _6, _7, _8, _9, _10, _11));
impl_concat_tuple!((_1), (_2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12));
impl_concat_tuple!((_1, _2), (_3));
impl_concat_tuple!((_1, _2), (_3, _4));
impl_concat_tuple!((_1, _2), (_3, _4, _5));
impl_concat_tuple!((_1, _2), (_3, _4, _5, _6));
impl_concat_tuple!((_1, _2), (_3, _4, _5, _6, _7));
impl_concat_tuple!((_1, _2), (_3, _4, _5, _6, _7, _8));
impl_concat_tuple!((_1, _2), (_3, _4, _5, _6, _7, _8, _9));
impl_concat_tuple!((_1, _2), (_3, _4, _5, _6, _7, _8, _9, _10));
impl_concat_tuple!((_1, _2), (_3, _4, _5, _6, _7, _8, _9, _10, _11));
impl_concat_tuple!((_1, _2), (_3, _4, _5, _6, _7, _8, _9, _10, _11, _12));
impl_concat_tuple!((_1, _2, _3), (_4));
impl_concat_tuple!((_1, _2, _3), (_4, _5));
impl_concat_tuple!((_1, _2, _3), (_4, _5, _6));
impl_concat_tuple!((_1, _2, _3), (_4, _5, _6, _7));
impl_concat_tuple!((_1, _2, _3), (_4, _5, _6, _7, _8));
impl_concat_tuple!((_1, _2, _3), (_4, _5, _6, _7, _8, _9));
impl_concat_tuple!((_1, _2, _3), (_4, _5, _6, _7, _8, _9, _10));
impl_concat_tuple!((_1, _2, _3), (_4, _5, _6, _7, _8, _9, _10, _11));
impl_concat_tuple!((_1, _2, _3), (_4, _5, _6, _7, _8, _9, _10, _11, _12));
impl_concat_tuple!((_1, _2, _3, _4), (_5));
impl_concat_tuple!((_1, _2, _3, _4), (_5, _6));
impl_concat_tuple!((_1, _2, _3, _4), (_5, _6, _7));
impl_concat_tuple!((_1, _2, _3, _4), (_5, _6, _7, _8));
impl_concat_tuple!((_1, _2, _3, _4), (_5, _6, _7, _8, _9));
impl_concat_tuple!((_1, _2, _3, _4), (_5, _6, _7, _8, _9, _10));
impl_concat_tuple!((_1, _2, _3, _4), (_5, _6, _7, _8, _9, _10, _11));
impl_concat_tuple!((_1, _2, _3, _4), (_5, _6, _7, _8, _9, _10, _11, _12));
impl_concat_tuple!((_1, _2, _3, _4, _5), (_6));
impl_concat_tuple!((_1, _2, _3, _4, _5), (_6, _7));
impl_concat_tuple!((_1, _2, _3, _4, _5), (_6, _7, _8));
impl_concat_tuple!((_1, _2, _3, _4, _5), (_6, _7, _8, _9));
impl_concat_tuple!((_1, _2, _3, _4, _5), (_6, _7, _8, _9, _10));
impl_concat_tuple!((_1, _2, _3, _4, _5), (_6, _7, _8, _9, _10, _11));
impl_concat_tuple!((_1, _2, _3, _4, _5), (_6, _7, _8, _9, _10, _11, _12));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6), (_7));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6), (_7, _8));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6), (_7, _8, _9));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6), (_7, _8, _9, _10));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6), (_7, _8, _9, _10, _11));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6), (_7, _8, _9, _10, _11, _12));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7), (_8));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7), (_8, _9));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7), (_8, _9, _10));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7), (_8, _9, _10, _11));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7), (_8, _9, _10, _11, _12));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7, _8), (_9));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7, _8), (_9, _10));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7, _8), (_9, _10, _11));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7, _8), (_9, _10, _11, _12));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7, _8, _9), (_10));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7, _8, _9), (_10, _11));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7, _8, _9), (_10, _11, _12));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7, _8, _9, _10), (_11));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7, _8, _9, _10), (_11, _12));
impl_concat_tuple!((_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11), (_12));

/// A [`Parse`] implementor created by `+`.
#[derive(Debug, Clone, Copy)]
pub struct Add<Lhs, Rhs> {
  lhs: Lhs,
  rhs: Rhs,
}

impl<Lhs, Rhs> Add<Lhs, Rhs> {
  /// Create a new instance with the left-hand side and right-hand side.
  #[inline]
  pub const fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self { lhs, rhs }
  }
}

impl<Lhs: Parse<Value: Concat<Rhs::Value>>, Rhs: Parse<State = Lhs::State, Heap = Lhs::Heap>> Parse
  for Add<Lhs, Rhs>
{
  type Value = <Lhs::Value as Concat<Rhs::Value>>::Output;
  type State = Lhs::State;
  type Heap = Lhs::Heap;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<'text, Self::Value>> {
    self.lhs.parse(input).and_then(|output| {
      input
        .reload(output.rest)
        .and_then(|mut input| self.rhs.parse(&mut input))
        .map(|rhs_output| Output {
          value: output.value.concat(rhs_output.value),
          rest: rhs_output.rest,
        })
    })
  }
}

impl<Lhs: Parse<Value: Concat<Rhs::Value>>, Rhs: Parse<State = Lhs::State, Heap = Lhs::Heap>>
  ops::Add<Combinator<Rhs>> for Combinator<Lhs>
{
  type Output = Combinator<Add<Lhs, Rhs>>;

  /// Create a new combinator to parse with the left-hand side, then parse with the right-hand side.
  /// The combinator will return the output with [`Concat`]-ed value and the sum of the digested,
  /// or reject if any of the parses rejects.
  #[inline]
  fn add(self, rhs: Combinator<Rhs>) -> Self::Output {
    Self::Output::new(Add::new(self.parser, rhs.parser))
  }
}

impl<Lhs: Parse<Value: Concat<()>>> ops::Add<char> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatChar<Lhs::State, Lhs::Heap>>>;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn add(self, rhs: char) -> Self::Output {
    Self::Output::new(Add::new(self.parser, EatChar::new(rhs)))
  }
}

impl<Lhs: Parse<Value: Concat<()>>> ops::Add<usize> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatUsize<Lhs::State, Lhs::Heap>>>;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn add(self, rhs: usize) -> Self::Output {
    Self::Output::new(Add::new(self.parser, EatUsize::new(rhs)))
  }
}

impl<Lhs: Parse<Value: Concat<()>>> ops::Add<String> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatString<Lhs::State, Lhs::Heap>>>;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn add(self, rhs: String) -> Self::Output {
    Self::Output::new(Add::new(self.parser, EatString::new(rhs)))
  }
}

impl<'a, Lhs: Parse<Value: Concat<()>>> ops::Add<&'a str> for Combinator<Lhs> {
  type Output = Combinator<Add<Lhs, EatStr<'a, Lhs::State, Lhs::Heap>>>;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn add(self, rhs: &'a str) -> Self::Output {
    Self::Output::new(Add::new(self.parser, EatStr::new(rhs)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::{wrap, Input};

  #[test]
  fn combinator_add() {
    let rejecter = || wrap(|_| Option::<Output<()>>::None);
    let accepter_unit = || {
      wrap(|input| {
        Some(Output {
          value: (),
          rest: &input.rest()[1..],
        })
      })
    };
    let accepter_int = || {
      wrap(|input| {
        Some(Output {
          value: (123,),
          rest: &input.rest()[1..],
        })
      })
    };

    // reject then accept, should return None
    assert!((rejecter() + accepter_unit())
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // accept then reject, should return None
    assert!((accepter_unit() + rejecter())
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // accept then accept, should return the sum of the digested
    // with the concat value
    assert_eq!(
      (accepter_unit() + accepter_int())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (123,),
        rest: "3",
      })
    );
    assert_eq!(
      (accepter_int() + accepter_unit())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (123,),
        rest: "3",
      })
    );
    assert_eq!(
      (accepter_int() + accepter_int()).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (123, 123),
        rest: "3",
      })
    );
  }

  #[test]
  fn combinator_add_char() {
    let eat1 = || {
      wrap(|input| {
        Some(Output {
          value: (),
          rest: &input.rest()[1..],
        })
      })
    };

    assert_eq!(
      (eat1() + '2')
        .parse(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
  }

  #[test]
  fn combinator_add_string() {
    let eat1 = || {
      wrap(|input| {
        Some(Output {
          value: (),
          rest: &input.rest()[1..],
        })
      })
    };

    assert_eq!(
      (eat1() + "23".to_string())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
  }

  #[test]
  fn combinator_add_str() {
    let eat1 = || {
      wrap(|input| {
        Some(Output {
          value: (),
          rest: &input.rest()[1..],
        })
      })
    };

    assert_eq!(
      (eat1() + "23")
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
  }

  #[test]
  fn combinator_add_usize() {
    let eat1 = || {
      wrap(|input| {
        Some(Output {
          value: (),
          rest: &input.rest()[1..],
        })
      })
    };

    // normal
    assert_eq!(
      (eat1() + 2)
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // overflow
    assert_eq!(
      (eat1() + 3)
        .parse(&mut Input::new("1", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      None
    );
    // 0
    assert_eq!(
      (eat1() + 0)
        .parse(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("2")
    );
  }
}
