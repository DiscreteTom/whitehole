use super::{
  create_closure_decorator, create_simple_decorator, create_value_decorator, AcceptedContext,
};
use crate::{
  combinator::{Action, Combinator, Input, Output},
  range::WithRange,
};
use core::fmt;
use std::marker::PhantomData;

create_closure_decorator!(Map, "See [`Combinator::map`].");
create_simple_decorator!(Tuple, "See [`Combinator::tuple`].");
create_value_decorator!(Bind, "See [`Combinator::bind`].");
create_closure_decorator!(Select, "See [`Combinator::select`].");
create_simple_decorator!(Range, "See [`Combinator::range`].");
create_simple_decorator!(Pop, "See [`Combinator::pop`].");

/// See [`Combinator::bind_default`].
pub struct BindDefault<T, D> {
  action: T,
  inner: PhantomData<D>,
}

impl<T: Copy, D> Copy for BindDefault<T, D> {}
impl<T: Clone, D> Clone for BindDefault<T, D> {
  fn clone(&self) -> Self {
    Self {
      action: self.action.clone(),
      inner: PhantomData,
    }
  }
}
impl<T: fmt::Debug, D> fmt::Debug for BindDefault<T, D> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let action = &self.action;
    f.debug_struct(stringify!(BindDefault))
      .field(stringify!(action), action)
      .finish()
  }
}

impl<T, D> BindDefault<T, D> {
  #[inline]
  const fn new(action: T) -> Self {
    Self {
      action,
      inner: PhantomData,
    }
  }
}

unsafe impl<State, Heap, NewValue, T: Action<State, Heap>, D: Fn(T::Value) -> NewValue>
  Action<State, Heap> for Map<T, D>
{
  type Value = NewValue;

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(input)
      .map(|output| output.map(&self.inner))
  }
}

unsafe impl<State, Heap, T: Action<State, Heap>> Action<State, Heap> for Tuple<T> {
  type Value = (T::Value,);

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.action.exec(input).map(|output| output.map(|v| (v,)))
  }
}

unsafe impl<State, Heap, T: Action<State, Heap>, D: Clone> Action<State, Heap> for Bind<T, D> {
  type Value = D;

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(input)
      .map(|output| output.map(|_| self.inner.clone()))
  }
}

unsafe impl<State, Heap, T: Action<State, Heap>, D: Default> Action<State, Heap>
  for BindDefault<T, D>
{
  type Value = D;

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(input)
      .map(|output| output.map(|_| Default::default()))
  }
}

unsafe impl<
    State,
    Heap,
    NewValue,
    T: Action<State, Heap>,
    D: Fn(AcceptedContext<Input<&str, &mut State, &mut Heap>, Output<T::Value>>) -> NewValue,
  > Action<State, Heap> for Select<T, D>
{
  type Value = NewValue;

  #[inline]
  fn exec(&self, mut input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.action.exec(input.reborrow()).map(|output| {
      unsafe { input.digest_unchecked(output.digested) }
        .map(|_| (self.inner)(AcceptedContext { input, output }))
    })
  }
}

unsafe impl<State, Heap, T: Action<State, Heap>> Action<State, Heap> for Range<T> {
  type Value = WithRange<T::Value>;

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    let start = input.instant().digested();
    self.action.exec(input).map(|output| {
      let digested = output.digested;
      output.map(|data| WithRange {
        range: start..start + digested,
        data,
      })
    })
  }
}

unsafe impl<State, Heap, V, T: Action<State, Heap, Value = (V,)>> Action<State, Heap> for Pop<T> {
  type Value = V;

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.action.exec(input).map(|output| output.map(|(v,)| v))
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
  pub fn map<State, Heap, NewValue, F: Fn(T::Value) -> NewValue>(
    self,
    mapper: F,
  ) -> Combinator<Map<T, F>>
  where
    T: Action<State, Heap>,
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

  /// Create a new combinator to set [`Output::value`] to the provided value.
  ///
  /// If your `Value` doesn't implement the [`Clone`] trait, consider using [`Self::select`] instead.
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
  pub fn bind<NewValue>(self, value: NewValue) -> Combinator<Bind<T, NewValue>>
  where
    NewValue: Clone,
  {
    Combinator::new(Bind::new(self.action, value))
  }

  /// Create a new combinator to set [`Output::value`] to the default value.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # fn t(combinator: Combinator<impl Action>) -> Combinator<impl Action<Value=i32>> {
  /// combinator.bind_default()
  /// # }
  /// ```
  #[inline]
  pub fn bind_default<NewValue>(self) -> Combinator<BindDefault<T, NewValue>>
  where
    NewValue: Default,
  {
    Combinator::new(BindDefault::new(self.action))
  }

  /// Create a new combinator to set [`Output::value`] by the `selector`.
  ///
  /// Use this if you need to calculate the value based on the [`Input`] and [`Output`].
  /// You can consume the original [`Output`] in the `selector`.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyValue(i32);
  /// # fn t(combinator: Combinator<impl Action>) {
  /// combinator.select(|ctx| MyValue(ctx.content().parse().unwrap()))
  /// # ;}
  /// ```
  #[inline]
  pub fn select<
    State,
    Heap,
    NewValue,
    F: Fn(AcceptedContext<Input<&str, &mut State, &mut Heap>, Output<T::Value>>) -> NewValue,
  >(
    self,
    selector: F,
  ) -> Combinator<Select<T, F>>
  where
    T: Action<State, Heap>,
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::wrap, instant::Instant};

  #[test]
  fn combinator_map() {
    assert_eq!(
      wrap(|input| input.digest(1).map(|output| output.map(|_| 1)))
        .map(Some)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: Some(1),
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_tuple() {
    assert_eq!(
      wrap(|input| input.digest(1).map(|output| output.map(|_| 1)))
        .tuple()
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (1,),
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_pop() {
    assert_eq!(
      wrap(|input| input.digest(1).map(|output| output.map(|_| 1)))
        .tuple()
        .pop()
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: 1,
        digested: 1
      })
    );
  }

  #[test]
  fn combinator_bind() {
    assert_eq!(
      wrap(|input| input.digest(1)).bind(123).exec(Input::new(
        Instant::new("123"),
        &mut (),
        &mut ()
      )),
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
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
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
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
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
        .exec(Input::new(Instant::new("123"), &mut (), &mut ())),
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
