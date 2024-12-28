use super::AcceptedContext;
use crate::{
  combinator::{wrap_unchecked, Action, Combinator, Input, Output},
  range::WithRange,
  C,
};

impl<T: Action> Combinator<T> {
  /// Create a new combinator to convert [`Output::value`] to a new value.
  ///
  /// You can consume the original [`Output::value`] in the `mapper`.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # fn t(combinator: C!()) {
  /// combinator.map(|value| Some(value))
  /// # ;}
  /// ```
  #[inline]
  pub fn map<NewValue>(self, mapper: impl Fn(T::Value) -> NewValue) -> C!(NewValue, @T) {
    unsafe { wrap_unchecked(move |input| self.exec(input).map(|output| output.map(&mapper))) }
  }

  /// Create a new combinator to wrap [`Output::value`] in an one-element tuple.
  ///
  /// This is useful when you use `+` to combine multiple combinators.
  /// See [`ops::add`](crate::combinator::ops::add) for more details.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # fn t(combinator: C!()) {
  /// combinator.tuple()
  /// # ;}
  /// ```
  #[inline]
  pub fn tuple(self) -> C!((T::Value,), @T) {
    self.map(|value| (value,))
  }

  /// Create a new combinator to set [`Output::value`] to the provided value.
  ///
  /// If your `Value` doesn't implement the [`Clone`] trait, consider using [`Self::select`] instead.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # #[derive(Clone)]
  /// # struct MyValue;
  /// # fn t(combinator: C!()) {
  /// combinator.bind(MyValue)
  /// # ;}
  /// ```
  #[inline]
  pub fn bind<NewValue>(self, value: NewValue) -> C!(NewValue, @T)
  where
    NewValue: Clone,
  {
    self.map(move |_| value.clone())
  }

  /// Create a new combinator to set [`Output::value`] to the default value.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # fn t(combinator: C!()) -> C!(i32) {
  /// combinator.bind_default()
  /// # }
  /// ```
  #[inline]
  pub fn bind_default<NewValue>(self) -> C!(NewValue, @T)
  where
    NewValue: Default,
  {
    self.map(|_| Default::default())
  }

  /// Create a new combinator to set [`Output::value`] by the `selector`.
  ///
  /// Use this if you need to calculate the value based on the [`Input`] and [`Output`].
  /// You can consume the original [`Output`] in the `selector`.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # struct MyValue(i32);
  /// # fn t(combinator: C!()) {
  /// combinator.select(|ctx| MyValue(ctx.content().parse().unwrap()))
  /// # ;}
  /// ```
  #[inline]
  pub fn select<NewValue>(
    self,
    selector: impl for<'text> Fn(
      AcceptedContext<Input<'text, &mut T::State, &mut T::Heap>, Output<T::Value>>,
    ) -> NewValue,
  ) -> C!(NewValue, @T) {
    unsafe {
      wrap_unchecked(move |mut input| {
        self.exec(input.reborrow()).map(|output| {
          input
            .digest_unchecked(output.digested)
            .map(|_| selector(AcceptedContext { input, output }))
        })
      })
    }
  }

  /// Create a new combinator to wrap [`Output::value`] in [`WithRange`]
  /// which includes the byte range of the digested text.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # fn t(combinator: C!()) {
  /// combinator.range()
  /// # ;}
  #[inline]
  pub fn range(self) -> C!(WithRange<T::Value>, @T) {
    self.select(|ctx| WithRange {
      range: ctx.range(),
      data: ctx.take().value,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::wrap;

  #[test]
  fn combinator_map() {
    assert_eq!(
      wrap(|input| input.digest(1).map(|output| output.map(|_| 1)))
        .map(Some)
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: Some(1),
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_bind() {
    assert_eq!(
      wrap(|input| input.digest(1))
        .bind(123)
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 123,
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_bind_default() {
    assert_eq!(
      wrap(|input| input.digest(1))
        .bind_default::<i32>()
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 0,
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_select() {
    assert_eq!(
      wrap(|input| input.digest(1))
        .select(|ctx| if ctx.content() == "1" { 1 } else { 2 })
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: 1,
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_range() {
    assert_eq!(
      wrap(|input| input.digest(1))
        .range()
        .exec(Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: WithRange {
          data: (),
          range: 0..1
        },
        digested: 1
      })
    );
  }
}
