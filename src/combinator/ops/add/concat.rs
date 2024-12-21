/// A helper trait to concat types when performing `+` on [`Combinator`](crate::combinator::Combinator)s.
/// See [`ops::add`](crate::combinator::ops::add) for more information.
///
/// Built-in implementations:
/// - `T.concat(()) -> T`
/// - `().concat((T1, T2, ...)) -> (T1, T2, ...)` for results with up to 12 elements.
/// - `(T1, T2, ...).concat((U1, U2, ...)) -> (T1, T2, ..., U1, U2, ...)`
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn concat() {
    // any type concat with () should return the type itself
    assert_eq!(().concat(()), ());
    assert_eq!(123.concat(()), 123);

    // unit tuple concat with a tuple should return the tuple
    assert_eq!(().concat((1,)), (1,));
    assert_eq!(().concat((1, 2)), (1, 2));

    // concat two non-unit tuples
    assert_eq!((1,).concat((2,)), (1, 2));
    assert_eq!((1,).concat((2, 3)), (1, 2, 3));
    assert_eq!((1,).concat((2, 3, 4)), (1, 2, 3, 4));
    assert_eq!((1,).concat((2, 3, 4, 5)), (1, 2, 3, 4, 5));
    assert_eq!((1,).concat((2, 3, 4, 5, 6)), (1, 2, 3, 4, 5, 6));
    assert_eq!((1,).concat((2, 3, 4, 5, 6, 7)), (1, 2, 3, 4, 5, 6, 7));
    assert_eq!((1,).concat((2, 3, 4, 5, 6, 7, 8)), (1, 2, 3, 4, 5, 6, 7, 8));
    assert_eq!(
      (1,).concat((2, 3, 4, 5, 6, 7, 8, 9)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9)
    );
    assert_eq!(
      (1,).concat((2, 3, 4, 5, 6, 7, 8, 9, 10)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
    );
    assert_eq!(
      (1,).concat((2, 3, 4, 5, 6, 7, 8, 9, 10, 11)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
    );
    assert_eq!(
      (1,).concat((2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    );
    assert_eq!((1, 2).concat((3,)), (1, 2, 3));
    assert_eq!((1, 2).concat((3, 4)), (1, 2, 3, 4));
    assert_eq!((1, 2).concat((3, 4, 5)), (1, 2, 3, 4, 5));
    assert_eq!((1, 2).concat((3, 4, 5, 6)), (1, 2, 3, 4, 5, 6));
    assert_eq!((1, 2).concat((3, 4, 5, 6, 7)), (1, 2, 3, 4, 5, 6, 7));
    assert_eq!((1, 2).concat((3, 4, 5, 6, 7, 8)), (1, 2, 3, 4, 5, 6, 7, 8));
    assert_eq!(
      (1, 2).concat((3, 4, 5, 6, 7, 8, 9)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9)
    );
    assert_eq!(
      (1, 2).concat((3, 4, 5, 6, 7, 8, 9, 10)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
    );
    assert_eq!(
      (1, 2).concat((3, 4, 5, 6, 7, 8, 9, 10, 11)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
    );
    assert_eq!(
      (1, 2).concat((3, 4, 5, 6, 7, 8, 9, 10, 11, 12)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    );
    assert_eq!((1, 2, 3).concat((4,)), (1, 2, 3, 4));
    assert_eq!((1, 2, 3).concat((4, 5)), (1, 2, 3, 4, 5));
    assert_eq!((1, 2, 3).concat((4, 5, 6)), (1, 2, 3, 4, 5, 6));
    assert_eq!((1, 2, 3).concat((4, 5, 6, 7)), (1, 2, 3, 4, 5, 6, 7));
    assert_eq!((1, 2, 3).concat((4, 5, 6, 7, 8)), (1, 2, 3, 4, 5, 6, 7, 8));
    assert_eq!(
      (1, 2, 3).concat((4, 5, 6, 7, 8, 9)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9)
    );
    assert_eq!(
      (1, 2, 3).concat((4, 5, 6, 7, 8, 9, 10)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
    );
    assert_eq!(
      (1, 2, 3).concat((4, 5, 6, 7, 8, 9, 10, 11)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
    );
    assert_eq!(
      (1, 2, 3).concat((4, 5, 6, 7, 8, 9, 10, 11, 12)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    );
    assert_eq!((1, 2, 3, 4).concat((5,)), (1, 2, 3, 4, 5));
    assert_eq!((1, 2, 3, 4).concat((5, 6)), (1, 2, 3, 4, 5, 6));
    assert_eq!((1, 2, 3, 4).concat((5, 6, 7)), (1, 2, 3, 4, 5, 6, 7));
    assert_eq!((1, 2, 3, 4).concat((5, 6, 7, 8)), (1, 2, 3, 4, 5, 6, 7, 8));
    assert_eq!(
      (1, 2, 3, 4).concat((5, 6, 7, 8, 9)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9)
    );
    assert_eq!(
      (1, 2, 3, 4).concat((5, 6, 7, 8, 9, 10)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
    );
    assert_eq!(
      (1, 2, 3, 4).concat((5, 6, 7, 8, 9, 10, 11)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
    );
    assert_eq!(
      (1, 2, 3, 4).concat((5, 6, 7, 8, 9, 10, 11, 12)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    );
    assert_eq!((1, 2, 3, 4, 5).concat((6,)), (1, 2, 3, 4, 5, 6));
    assert_eq!((1, 2, 3, 4, 5).concat((6, 7)), (1, 2, 3, 4, 5, 6, 7));
    assert_eq!((1, 2, 3, 4, 5).concat((6, 7, 8)), (1, 2, 3, 4, 5, 6, 7, 8));
    assert_eq!(
      (1, 2, 3, 4, 5).concat((6, 7, 8, 9)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9)
    );
    assert_eq!(
      (1, 2, 3, 4, 5).concat((6, 7, 8, 9, 10)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
    );
    assert_eq!(
      (1, 2, 3, 4, 5).concat((6, 7, 8, 9, 10, 11)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
    );
    assert_eq!(
      (1, 2, 3, 4, 5).concat((6, 7, 8, 9, 10, 11, 12)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    );
    assert_eq!((1, 2, 3, 4, 5, 6).concat((7,)), (1, 2, 3, 4, 5, 6, 7));
    assert_eq!((1, 2, 3, 4, 5, 6).concat((7, 8)), (1, 2, 3, 4, 5, 6, 7, 8));
    assert_eq!(
      (1, 2, 3, 4, 5, 6).concat((7, 8, 9)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6).concat((7, 8, 9, 10)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6).concat((7, 8, 9, 10, 11)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6).concat((7, 8, 9, 10, 11, 12)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    );
    assert_eq!((1, 2, 3, 4, 5, 6, 7).concat((8,)), (1, 2, 3, 4, 5, 6, 7, 8));
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7).concat((8, 9)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7).concat((8, 9, 10)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7).concat((8, 9, 10, 11)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7).concat((8, 9, 10, 11, 12)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7, 8).concat((9,)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7, 8).concat((9, 10)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7, 8).concat((9, 10, 11)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7, 8).concat((9, 10, 11, 12)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7, 8, 9).concat((10,)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7, 8, 9).concat((10, 11)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7, 8, 9).concat((10, 11, 12)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10).concat((11,)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10).concat((11, 12)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    );
    assert_eq!(
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11).concat((12,)),
      (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)
    );
  }

  #[test]
  fn use_concat() {
    assert_eq!(().concat((123,)).concat(()), (123,));
    assert_eq!((123,).concat(()).concat((123,)), (123, 123));
  }
}
