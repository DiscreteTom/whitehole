use super::{create_closure_decorator, create_simple_decorator, create_value_decorator, Accepted};
use crate::{
  action::Context,
  combinator::{Action, Combinator, Output},
  instant::Instant,
  range::WithRange,
};

create_closure_decorator!(Map, "See [`Combinator::map`].");
create_simple_decorator!(Tuple, "See [`Combinator::tuple`].");
create_value_decorator!(Bind, "See [`Combinator::bind`].");
create_closure_decorator!(BindWith, "See [`Combinator::bind_with`].");
create_closure_decorator!(Select, "See [`Combinator::select`].");
create_simple_decorator!(Range, "See [`Combinator::range`].");
create_simple_decorator!(Pop, "See [`Combinator::pop`].");

unsafe impl<
    Text: ?Sized,
    State,
    Heap,
    NewValue,
    T: Action<Text, State, Heap>,
    D: Fn(T::Value) -> NewValue,
  > Action<Text, State, Heap> for Map<T, D>
{
  type Value = NewValue;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(instant, ctx)
      .map(|output| output.map(&self.inner))
  }
}

unsafe impl<Text: ?Sized, State, Heap, T: Action<Text, State, Heap>> Action<Text, State, Heap>
  for Tuple<T>
{
  type Value = (T::Value,);

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(instant, ctx)
      .map(|output| output.map(|v| (v,)))
  }
}

unsafe impl<Text: ?Sized, State, Heap, T: Action<Text, State, Heap>, D: Clone>
  Action<Text, State, Heap> for Bind<T, D>
{
  type Value = D;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(instant, ctx)
      .map(|output| output.map(|_| self.inner.clone()))
  }
}

unsafe impl<Text: ?Sized, State, Heap, T: Action<Text, State, Heap>, NewValue, D: Fn() -> NewValue>
  Action<Text, State, Heap> for BindWith<T, D>
{
  type Value = NewValue;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(instant, ctx)
      .map(|output| output.map(|_| (self.inner)()))
  }
}

unsafe impl<
    Text: ?Sized,
    State,
    Heap,
    NewValue,
    T: Action<Text, State, Heap>,
    D: Fn(Accepted<&Text, T::Value>, Context<&mut State, &mut Heap>) -> NewValue,
  > Action<Text, State, Heap> for Select<T, D>
{
  type Value = NewValue;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    mut ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(instant, ctx.reborrow())
      .map(|output| Output {
        digested: output.digested,
        value: (self.inner)(Accepted::new(instant, output), ctx),
      })
  }
}

unsafe impl<Text: ?Sized, State, Heap, T: Action<Text, State, Heap>> Action<Text, State, Heap>
  for Range<T>
{
  type Value = WithRange<T::Value>;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    let start = instant.digested();
    self.action.exec(instant, ctx).map(|output| {
      let digested = output.digested;
      output.map(|data| WithRange {
        range: start..start + digested,
        data,
      })
    })
  }
}

unsafe impl<Text: ?Sized, State, Heap, V, T: Action<Text, State, Heap, Value = (V,)>>
  Action<Text, State, Heap> for Pop<T>
{
  type Value = V;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(instant, ctx)
      .map(|output| output.map(|(v,)| v))
  }
}

impl<T> Combinator<T> {
  /// Create a new combinator to convert [`Output::value`] to a new value.
  ///
  /// You can consume the original [`Output::value`] in the `mapper`.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # fn t(combinator: Combinator<impl Action>) {
  /// combinator.map(|value| Some(value))
  /// # ;}
  /// ```
  #[inline]
  pub fn map<Text: ?Sized, State, Heap, NewValue, F: Fn(T::Value) -> NewValue>(
    self,
    mapper: F,
  ) -> Combinator<Map<T, F>>
  where
    T: Action<Text, State, Heap>,
  {
    Combinator::new(Map::new(self.action, mapper))
  }

  /// Create a new combinator to wrap [`Output::value`] in an one-element tuple.
  ///
  /// This is useful when you use `+` to combine multiple combinators.
  /// See [`ops::add`](crate::combinator::ops::add) for more details.
  ///
  /// The reverse operation is [`Self::pop`].
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # fn t(combinator: Combinator<impl Action>) {
  /// combinator.tuple()
  /// # ;}
  /// ```
  #[inline]
  pub fn tuple(self) -> Combinator<Tuple<T>> {
    Combinator::new(Tuple::new(self.action))
  }

  /// Create a new combinator to take the value from an one-element tuple as [`Output::value`].
  ///
  /// This is reverse to [`Self::tuple`].
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # fn t(combinator: Combinator<impl Action>) {
  /// combinator.pop()
  /// # ;}
  /// ```
  #[inline]
  pub fn pop(self) -> Combinator<Pop<T>> {
    Combinator::new(Pop::new(self.action))
  }

  /// Create a new combinator to set [`Output::value`] to the provided clone-able value.
  ///
  /// If your value doesn't implement the [`Clone`] trait, consider using [`Self::bind_with`] or [`Self::select`] instead.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # #[derive(Clone)]
  /// # struct MyValue;
  /// # fn t(combinator: Combinator<impl Action>) {
  /// combinator.bind(MyValue)
  /// # ;}
  /// ```
  #[inline]
  pub fn bind<NewValue: Clone>(self, value: NewValue) -> Combinator<Bind<T, NewValue>> {
    Combinator::new(Bind::new(self.action, value))
  }

  /// Create a new combinator to set [`Output::value`] with the provided factory.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # fn t(combinator: Combinator<impl Action>) -> Combinator<impl Action<Value=i32>> {
  /// combinator.bind_with(|| 0i32)
  /// # }
  /// ```
  #[inline]
  pub fn bind_with<NewValue, Factory: Fn() -> NewValue>(
    self,
    factory: Factory,
  ) -> Combinator<BindWith<T, Factory>> {
    Combinator::new(BindWith::new(self.action, factory))
  }

  /// Create a new combinator to set [`Output::value`] by the `selector`.
  ///
  /// Use this if you need to calculate the value based on the [`Instant`], [`Context`] and [`Output`].
  /// You can consume the original [`Output`] in the `selector`.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyValue(i32);
  /// # fn t(combinator: Combinator<impl Action>) {
  /// combinator.select(|accept, _| MyValue(accept.content().parse().unwrap()))
  /// # ;}
  /// ```
  #[inline]
  pub fn select<
    Text: ?Sized,
    State,
    Heap,
    NewValue,
    F: Fn(Accepted<&Text, T::Value>, Context<&mut State, &mut Heap>) -> NewValue,
  >(
    self,
    selector: F,
  ) -> Combinator<Select<T, F>>
  where
    T: Action<Text, State, Heap>,
  {
    Combinator::new(Select::new(self.action, selector))
  }

  /// Create a new combinator to wrap [`Output::value`] in [`WithRange`]
  /// which includes the byte range of the digested text.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # fn t(combinator: Combinator<impl Action>) {
  /// combinator.range()
  /// # ;}
  #[inline]
  pub fn range(self) -> Combinator<Range<T>> {
    Combinator::new(Range::new(self.action))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{bytes, wrap},
    instant::Instant,
  };

  #[test]
  fn combinator_map() {
    assert_eq!(
      wrap(|instant, _| instant.accept(1).map(|output| output.map(|_| 1)))
        .map(Some)
        .exec(&Instant::new("123"), Context::default()),
      Some(Output {
        value: Some(1),
        digested: 1
      })
    );
    assert_eq!(
      bytes::wrap(|instant, _| instant.accept(1).map(|output| output.map(|_| 1)))
        .map(Some)
        .exec(&Instant::new(b"123"), Context::default()),
      Some(Output {
        value: Some(1),
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_tuple() {
    assert_eq!(
      wrap(|instant, _| instant.accept(1).map(|output| output.map(|_| 1)))
        .tuple()
        .exec(&Instant::new("123"), Context::default()),
      Some(Output {
        value: (1,),
        digested: 1
      })
    );
    assert_eq!(
      bytes::wrap(|instant, _| instant.accept(1).map(|output| output.map(|_| 1)))
        .tuple()
        .exec(&Instant::new(b"123"), Context::default()),
      Some(Output {
        value: (1,),
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_pop() {
    assert_eq!(
      wrap(|instant, _| instant.accept(1).map(|output| output.map(|_| 1)))
        .tuple()
        .pop()
        .exec(&Instant::new("123"), Context::default()),
      Some(Output {
        value: 1,
        digested: 1
      })
    );
    assert_eq!(
      bytes::wrap(|instant, _| instant.accept(1).map(|output| output.map(|_| 1)))
        .tuple()
        .pop()
        .exec(&Instant::new(b"123"), Context::default()),
      Some(Output {
        value: 1,
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_bind() {
    assert_eq!(
      wrap(|instant, _| instant.accept(1))
        .bind(123)
        .exec(&Instant::new("123"), Context::default()),
      Some(Output {
        value: 123,
        digested: 1
      })
    );
    assert_eq!(
      bytes::wrap(|instant, _| instant.accept(1))
        .bind(123)
        .exec(&Instant::new(b"123"), Context::default()),
      Some(Output {
        value: 123,
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_bind_with() {
    assert_eq!(
      wrap(|instant, _| instant.accept(1))
        .bind_with(|| 0i32)
        .exec(&Instant::new("123"), Context::default()),
      Some(Output {
        value: 0,
        digested: 1
      })
    );
    assert_eq!(
      bytes::wrap(|instant, _| instant.accept(1))
        .bind_with(|| 0i32)
        .exec(&Instant::new(b"123"), Context::default()),
      Some(Output {
        value: 0,
        digested: 1
      })
    );

    // make sure copy-able and clone-able
    let a = wrap::<(), (), _, _>(|instant, _| instant.accept(1)).bind_with(|| 0i32);
    let _ = a;
    let _ = a.clone();

    assert_eq!(
      format!("{:?}", a),
      "Combinator { action: BindWith { action: Wrap } }"
    );
  }

  #[test]
  fn combinator_select() {
    assert_eq!(
      wrap(|instant, _| instant.accept(1))
        .select(|accept, _| if accept.content() == "1" { 1 } else { 2 })
        .exec(&Instant::new("123"), Context::default()),
      Some(Output {
        value: 1,
        digested: 1
      })
    );
    assert_eq!(
      bytes::wrap(|instant, _| instant.accept(1))
        .select(|accept, _| if accept.content() == b"1" { 1 } else { 2 })
        .exec(&Instant::new(b"123"), Context::default()),
      Some(Output {
        value: 1,
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_range() {
    assert_eq!(
      wrap(|instant, _| instant.accept(1))
        .range()
        .exec(&Instant::new("123"), Context::default()),
      Some(Output {
        value: WithRange {
          data: (),
          range: 0..1
        },
        digested: 1
      })
    );
    assert_eq!(
      bytes::wrap(|instant, _| instant.accept(1))
        .range()
        .exec(&Instant::new(b"123"), Context::default()),
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
