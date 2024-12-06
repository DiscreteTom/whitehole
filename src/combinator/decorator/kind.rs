use super::AcceptedContext;
use crate::{
  combinator::{wrap, Combinator, Input, Output, Parse},
  Combinator,
};

impl<T: Parse> Combinator<T> {
  /// Create a new combinator to convert [`Output::kind`] to a new kind value.
  ///
  /// You can consume the original [`Output::kind`] in the `mapper`.
  /// # Examples
  /// ```
  /// # use whitehole::Combinator;
  /// # fn t(combinator: Combinator!()) {
  /// combinator.map(|kind| Some(kind))
  /// # ;}
  /// ```
  #[inline]
  pub fn map<NewKind>(self, mapper: impl Fn(T::Kind) -> NewKind) -> Combinator!(NewKind, @T) {
    wrap(move |input| self.parse(input).map(|output| output.map(&mapper)))
  }

  /// Create a new combinator to wrap [`Output::kind`] in a tuple.
  ///
  /// This is useful when you use `+` to combine multiple combinators.
  /// See [`Concat`](crate::combinator::ops::add::Concat) for more details.
  /// # Examples
  /// ```
  /// # use whitehole::Combinator;
  /// # fn t(combinator: Combinator!()) {
  /// combinator.map(|kind| Some(kind))
  /// # ;}
  /// ```
  #[inline]
  pub fn tuple(self) -> Combinator!((T::Kind,), @T) {
    self.map(|kind| (kind,))
  }

  /// Create a new combinator to set [`Output::kind`] to the provided kind value.
  ///
  /// If your `Kind` doesn't implement the [`Clone`] trait, consider using [`Self::select`] instead.
  /// # Examples
  /// ```
  /// # use whitehole::Combinator;
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn t(combinator: Combinator!()) {
  /// combinator.bind(MyKind::A)
  /// # ;}
  /// ```
  #[inline]
  pub fn bind<NewKind>(self, kind: NewKind) -> Combinator!(NewKind, @T)
  where
    NewKind: Clone,
  {
    self.map(move |_| kind.clone())
  }

  /// Create a new combinator to set [`Output::kind`] to the default kind value.
  /// # Examples
  /// ```
  /// # use whitehole::Combinator;
  /// # fn t(combinator: Combinator!()) -> Combinator!(i32) {
  /// combinator.bind_default()
  /// # }
  /// ```
  #[inline]
  pub fn bind_default<NewKind>(self) -> Combinator!(NewKind, @T)
  where
    NewKind: Default,
  {
    self.map(|_| NewKind::default())
  }

  /// Create a new combinator to set [`Output::kind`] by the `selector`.
  ///
  /// Use this if you need to calculate the kind value based on the [`Input`] and [`Output`].
  /// You can consume the original [`Output`] in the `selector`.
  /// # Examples
  /// ```
  /// # use whitehole::Combinator;
  /// # enum MyKind { Num(i32) }
  /// # fn t(combinator: Combinator!()) {
  /// combinator.select(|ctx| MyKind::Num(ctx.content().parse().unwrap()))
  /// # ;}
  /// ```
  #[inline]
  pub fn select<NewKind>(
    self,
    selector: impl for<'text> Fn(
      AcceptedContext<&mut Input<'text, &mut T::State, &mut T::Heap>, Output<'text, T::Kind>>,
    ) -> NewKind,
  ) -> Combinator!(NewKind, @T) {
    wrap(move |input| {
      self.parse(input).map(|output| Output {
        rest: output.rest,
        kind: selector(AcceptedContext { input, output }),
      })
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn combinator_map() {
    assert_eq!(
      wrap(|input| Some(Output {
        kind: 1,
        rest: &input.rest()[1..]
      }))
      .map(Some)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: Some(1),
        rest: "23"
      })
    );
  }

  #[test]
  fn combinator_bind() {
    assert_eq!(
      wrap(|input| Some(Output {
        kind: (),
        rest: &input.rest()[1..]
      }))
      .bind(123)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 123,
        rest: "23"
      })
    );
  }

  #[test]
  fn combinator_bind_default() {
    assert_eq!(
      wrap(|input| Some(Output {
        kind: (),
        rest: &input.rest()[1..]
      }))
      .bind_default::<i32>()
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        rest: "23"
      })
    );
  }

  #[test]
  fn combinator_select() {
    assert_eq!(
      wrap(|input| Some(Output {
        kind: (),
        rest: &input.rest()[1..]
      }))
      .select(|ctx| if ctx.content() == "1" { 1 } else { 2 })
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 1,
        rest: "23"
      })
    );
  }
}
