use super::{create_closure_decorator, AcceptedContext};
use crate::combinator::{Action, Combinator, Input, Output};

create_closure_decorator!(Prepare, "See [`Combinator::prepare`].");
create_closure_decorator!(Then, "See [`Combinator::then`].");
create_closure_decorator!(Catch, "See [`Combinator::catch`].");
create_closure_decorator!(Finally, "See [`Combinator::finally`].");

unsafe impl<T: Action, D: Fn(Input<&mut T::State, &mut T::Heap>)> Action for Prepare<T, D> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    (self.inner)(input.reborrow());
    self.action.exec(input)
  }
}

unsafe impl<T: Action, D: Fn(AcceptedContext<Input<&mut T::State, &mut T::Heap>, &Output<T::Value>>)>
  Action for Then<T, D>
{
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.action.exec(input.reborrow()).inspect(|output| {
      (self.inner)(AcceptedContext { input, output });
    })
  }
}

unsafe impl<T: Action, D: Fn(Input<&mut T::State, &mut T::Heap>)> Action for Catch<T, D> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let output = self.action.exec(input.reborrow());
    if output.is_none() {
      (self.inner)(input);
    }
    output
  }
}

unsafe impl<T: Action, D: Fn(Input<&mut T::State, &mut T::Heap>)> Action for Finally<T, D> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let output = self.action.exec(input.reborrow());
    (self.inner)(input);
    output
  }
}

impl<T: Action> Combinator<T> {
  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// before being executed.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: C!((), MyState)) {
  /// combinator.prepare(|input| input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn prepare<F: Fn(Input<&mut T::State, &mut T::Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Prepare<T, F>> {
    Combinator::new(Prepare::new(self.action, modifier))
  }

  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// after being accepted.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: C!((), MyState)) {
  /// combinator.then(|mut ctx| ctx.state().value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn then<F: Fn(AcceptedContext<Input<&mut T::State, &mut T::Heap>, &Output<T::Value>>)>(
    self,
    modifier: F,
  ) -> Combinator<Then<T, F>> {
    Combinator::new(Then::new(self.action, modifier))
  }

  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// after being rejected.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: C!((), MyState)) {
  /// combinator.catch(|input| input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn catch<F: Fn(Input<&mut T::State, &mut T::Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Catch<T, F>> {
    Combinator::new(Catch::new(self.action, modifier))
  }

  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// after the combinator is executed,
  /// no matter whether it is accepted or rejected.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: C!((), MyState)) {
  /// combinator.finally(|input| input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn finally<F: Fn(Input<&mut T::State, &mut T::Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Finally<T, F>> {
    Combinator::new(Finally::new(self.action, modifier))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::wrap, instant::Instant, C};

  #[derive(Debug, Default, PartialEq, Eq)]
  struct State {
    from: i32,
    to: i32,
  }

  fn accepter() -> C!((), State) {
    wrap(|input: Input<&mut State, &mut ()>| {
      input.state.to = input.state.from;
      input.digest(1)
    })
  }

  fn rejecter() -> C!((), State) {
    wrap(|input: Input<&mut State, &mut ()>| {
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
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()).unwrap())
      .is_some());
    assert_eq!(state, State { from: 1, to: 1 });
  }

  #[test]
  fn combinator_then() {
    let mut state = State::default();
    assert!(accepter()
      .then(|ctx| {
        ctx.input.state.from = 1;
      })
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()).unwrap())
      .is_some());
    assert_eq!(state, State { from: 1, to: 0 });

    let mut state = State::default();
    assert!(rejecter()
      .then(|ctx| {
        ctx.input.state.from = 1;
      })
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()).unwrap())
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
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()).unwrap())
      .is_some());
    assert_eq!(state, State { from: 0, to: 0 });

    let mut state = State::default();
    assert!(rejecter()
      .catch(|input| {
        input.state.from = 1;
      })
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()).unwrap())
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
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()).unwrap())
      .is_some());
    assert_eq!(state, State { from: 0, to: 1 });

    let mut state = State::default();
    assert!(rejecter()
      .finally(|input| {
        input.state.to = 1;
      })
      .exec(Input::new(Instant::new("123"), &mut state, &mut ()).unwrap())
      .is_none());
    assert_eq!(state, State { from: 0, to: 1 });
  }
}
