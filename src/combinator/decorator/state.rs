use super::{create_closure_decorator, AcceptedContext};
use crate::combinator::{Action, Combinator, Input, Output};

create_closure_decorator!(Prepare, "See [`Combinator::prepare`].");
create_closure_decorator!(Then, "See [`Combinator::then`].");
create_closure_decorator!(Catch, "See [`Combinator::catch`].");
create_closure_decorator!(Finally, "See [`Combinator::finally`].");

unsafe impl<
    Text: ?Sized,
    State,
    Heap,
    T: Action<Text, State, Heap>,
    D: Fn(Input<&Text, &mut State, &mut Heap>),
  > Action<Text, State, Heap> for Prepare<T, D>
{
  type Value = T::Value;

  #[inline]
  fn exec(&self, mut input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    (self.inner)(input.reborrow());
    self.action.exec(input)
  }
}

unsafe impl<
    Text: ?Sized,
    State,
    Heap,
    T: Action<Text, State, Heap>,
    D: Fn(AcceptedContext<Input<&Text, &mut State, &mut Heap>, &Output<T::Value>>),
  > Action<Text, State, Heap> for Then<T, D>
{
  type Value = T::Value;

  #[inline]
  fn exec(&self, mut input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.action.exec(input.reborrow()).inspect(|output| {
      (self.inner)(AcceptedContext::new(input, output));
    })
  }
}

unsafe impl<
    Text: ?Sized,
    State,
    Heap,
    T: Action<Text, State, Heap>,
    D: Fn(Input<&Text, &mut State, &mut Heap>),
  > Action<Text, State, Heap> for Catch<T, D>
{
  type Value = T::Value;

  #[inline]
  fn exec(&self, mut input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    let output = self.action.exec(input.reborrow());
    if output.is_none() {
      (self.inner)(input);
    }
    output
  }
}

unsafe impl<
    Text: ?Sized,
    State,
    Heap,
    T: Action<Text, State, Heap>,
    D: Fn(Input<&Text, &mut State, &mut Heap>),
  > Action<Text, State, Heap> for Finally<T, D>
{
  type Value = T::Value;

  #[inline]
  fn exec(&self, mut input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    let output = self.action.exec(input.reborrow());
    (self.inner)(input);
    output
  }
}

impl<T> Combinator<T> {
  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// before being executed.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: Combinator<impl Action<MyState>>) {
  /// combinator.prepare(|input| input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn prepare<Text: ?Sized, State, Heap, F: Fn(Input<&Text, &mut State, &mut Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Prepare<T, F>>
  where
    T: Action<Text, State, Heap>,
  {
    Combinator::new(Prepare::new(self.action, modifier))
  }

  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// after being accepted.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: Combinator<impl Action<MyState>>) {
  /// combinator.then(|mut ctx| ctx.state().value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn then<
    Text: ?Sized,
    State,
    Heap,
    F: Fn(AcceptedContext<Input<&Text, &mut State, &mut Heap>, &Output<T::Value>>),
  >(
    self,
    modifier: F,
  ) -> Combinator<Then<T, F>>
  where
    T: Action<Text, State, Heap>,
  {
    Combinator::new(Then::new(self.action, modifier))
  }

  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// after being rejected.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: Combinator<impl Action<MyState>>) {
  /// combinator.catch(|input| input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn catch<Text: ?Sized, State, Heap, F: Fn(Input<&Text, &mut State, &mut Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Catch<T, F>>
  where
    T: Action<Text, State, Heap>,
  {
    Combinator::new(Catch::new(self.action, modifier))
  }

  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// after the combinator is executed,
  /// no matter whether it is accepted or rejected.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: Combinator<impl Action<MyState>>) {
  /// combinator.finally(|input| input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn finally<Text: ?Sized, State, Heap, F: Fn(Input<&Text, &mut State, &mut Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Finally<T, F>>
  where
    T: Action<Text, State, Heap>,
  {
    Combinator::new(Finally::new(self.action, modifier))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::wrap, instant::Instant};

  #[derive(Debug, Default, PartialEq, Eq)]
  struct State {
    from: i32,
    to: i32,
  }

  fn accepter() -> Combinator<impl Action<str, State, Value = ()>> {
    wrap(|input: Input<&str, &mut State, &mut ()>| {
      input.state.to = input.state.from;
      input.digest(1)
    })
  }

  fn rejecter() -> Combinator<impl Action<str, State, Value = ()>> {
    wrap(|input: Input<&str, &mut State, &mut ()>| {
      input.state.to = input.state.from;
      None
    })
  }

  #[test]
  fn combinator_prepare() {
    let mut state = State::default();
    assert!(accepter()
      .prepare(|input| {
        input.state.from = 1;
      })
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()))
      .is_some());
    assert_eq!(state, State { from: 1, to: 1 });
  }

  #[test]
  fn combinator_then() {
    let mut state = State::default();
    assert!(accepter()
      .then(|mut ctx| {
        ctx.state().from = 1;
      })
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()))
      .is_some());
    assert_eq!(state, State { from: 1, to: 0 });

    let mut state = State::default();
    assert!(rejecter()
      .then(|mut ctx| {
        ctx.state().from = 1;
      })
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()))
      .is_none());
    assert_eq!(state, State { from: 0, to: 0 });
  }

  #[test]
  fn combinator_catch() {
    let mut state = State::default();
    assert!(accepter()
      .catch(|input| {
        input.state.from = 1;
      })
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()))
      .is_some());
    assert_eq!(state, State { from: 0, to: 0 });

    let mut state = State::default();
    assert!(rejecter()
      .catch(|input| {
        input.state.from = 1;
      })
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()))
      .is_none());
    assert_eq!(state, State { from: 1, to: 0 });
  }

  #[test]
  fn combinator_finally() {
    let mut state = State::default();
    assert!(accepter()
      .finally(|input| {
        input.state.to = 1;
      })
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()))
      .is_some());
    assert_eq!(state, State { from: 0, to: 1 });

    let mut state = State::default();
    assert!(rejecter()
      .finally(|input| {
        input.state.to = 1;
      })
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()))
      .is_none());
    assert_eq!(state, State { from: 0, to: 1 });
  }
}
