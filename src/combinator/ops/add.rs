//! Overload `+` operator for [`Combinator`].

use super::{EatChar, EatStr, EatString, EatUsize};
use crate::combinator::{Combinator, Input, Output, Parse};
use std::ops;

/// A helper trait to concat types when calling `+` on [`Combinator`]s.
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

/// A composite combinator created by `+`.
#[derive(Debug, Clone, Copy)]
pub struct Add<Lhs, Rhs> {
  lhs: Lhs,
  rhs: Rhs,
}

impl<Lhs, Rhs> Add<Lhs, Rhs> {
  #[inline]
  pub fn new(lhs: Lhs, rhs: Rhs) -> Self {
    Self { lhs, rhs }
  }
}

impl<State, Heap, Lhs: Parse<State, Heap, Kind: Concat<Rhs::Kind>>, Rhs: Parse<State, Heap>>
  Parse<State, Heap> for Add<Lhs, Rhs>
{
  type Kind = <Lhs::Kind as Concat<Rhs::Kind>>::Output;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    self.lhs.parse(input).and_then(|output| {
      input
        .reload(output.rest)
        .and_then(|mut input| self.rhs.parse(&mut input))
        .map(|rhs_output| Output {
          kind: output.kind.concat(rhs_output.kind),
          rest: rhs_output.rest,
        })
    })
  }
}

impl<State, Heap, Lhs: Parse<State, Heap, Kind: Concat<Rhs::Kind>>, Rhs: Parse<State, Heap>>
  ops::Add<Combinator<State, Heap, Rhs>> for Combinator<State, Heap, Lhs>
{
  type Output =
    Combinator<State, Heap, Add<Combinator<State, Heap, Lhs>, Combinator<State, Heap, Rhs>>>;

  /// Try to parse with the left-hand side, if it fails, try the right-hand side.
  #[inline]
  fn add(self, rhs: Combinator<State, Heap, Rhs>) -> Self::Output {
    Self::Output::new(Add::new(self, rhs))
  }
}

impl<State, Heap, Lhs: Parse<State, Heap, Kind: Concat<()>>> ops::Add<char>
  for Combinator<State, Heap, Lhs>
{
  type Output = Combinator<
    State,
    Heap,
    Add<Combinator<State, Heap, Lhs>, Combinator<State, Heap, EatChar<State, Heap>>>,
  >;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn add(self, rhs: char) -> Self::Output {
    Self::Output::new(Add::new(self, Combinator::new(EatChar::new(rhs))))
  }
}

impl<State, Heap, Lhs: Parse<State, Heap, Kind: Concat<()>>> ops::Add<usize>
  for Combinator<State, Heap, Lhs>
{
  type Output = Combinator<
    State,
    Heap,
    Add<Combinator<State, Heap, Lhs>, Combinator<State, Heap, EatUsize<State, Heap>>>,
  >;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn add(self, rhs: usize) -> Self::Output {
    Self::Output::new(Add::new(self, Combinator::new(EatUsize::new(rhs))))
  }
}

impl<State, Heap, Lhs: Parse<State, Heap, Kind: Concat<()>>> ops::Add<String>
  for Combinator<State, Heap, Lhs>
{
  type Output = Combinator<
    State,
    Heap,
    Add<Combinator<State, Heap, Lhs>, Combinator<State, Heap, EatString<State, Heap>>>,
  >;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn add(self, rhs: String) -> Self::Output {
    Self::Output::new(Add::new(self, Combinator::new(EatString::new(rhs))))
  }
}

impl<'a, State, Heap, Lhs: Parse<State, Heap, Kind: Concat<()>>> ops::Add<&'a str>
  for Combinator<State, Heap, Lhs>
{
  type Output = Combinator<
    State,
    Heap,
    Add<Combinator<State, Heap, Lhs>, Combinator<State, Heap, EatStr<'a, State, Heap>>>,
  >;

  /// Similar to `self + eat(rhs)`. See [`eat`](crate::combinator::eat).
  #[inline]
  fn add(self, rhs: &'a str) -> Self::Output {
    Self::Output::new(Add::new(self, Combinator::new(EatStr::new(rhs))))
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
          kind: (),
          rest: &input.rest()[1..],
        })
      })
    };
    let accepter_int = || {
      wrap(|input| {
        Some(Output {
          kind: (123,),
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
    // with the concat kind
    assert_eq!(
      (accepter_unit() + accepter_int())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (123,),
        rest: "3",
      })
    );
    assert_eq!(
      (accepter_int() + accepter_unit())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (123,),
        rest: "3",
      })
    );
    assert_eq!(
      (accepter_int() + accepter_int()).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (123, 123),
        rest: "3",
      })
    );
  }

  #[test]
  fn combinator_add_char() {
    let eat1 = || {
      wrap(|input| {
        Some(Output {
          kind: (),
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
          kind: (),
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
          kind: (),
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
          kind: (),
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
