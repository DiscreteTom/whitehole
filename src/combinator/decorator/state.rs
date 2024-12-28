use super::AcceptedContext;
use crate::{
  combinator::{wrap_unchecked, Action, Combinator, Input, Output},
  C,
};

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
  pub fn prepare(self, modifier: impl Fn(Input<&mut T::State, &mut T::Heap>)) -> C!(@T) {
    unsafe {
      wrap_unchecked(move |mut input| {
        modifier(input.reborrow());
        self.exec(input)
      })
    }
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
  pub fn then(
    self,
    modifier: impl for<'text> Fn(
      AcceptedContext<Input<'text, &mut T::State, &mut T::Heap>, &Output<T::Value>>,
    ),
  ) -> C!(@T) {
    unsafe {
      wrap_unchecked(move |mut input| {
        self.exec(input.reborrow()).inspect(|output| {
          modifier(AcceptedContext { input, output });
        })
      })
    }
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
  pub fn catch(self, modifier: impl Fn(Input<&mut T::State, &mut T::Heap>)) -> C!(@T) {
    unsafe {
      wrap_unchecked(move |mut input| {
        let output = self.exec(input.reborrow());
        if output.is_none() {
          modifier(input);
        }
        output
      })
    }
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
  pub fn finally(self, modifier: impl Fn(Input<&mut T::State, &mut T::Heap>)) -> C!(@T) {
    unsafe {
      wrap_unchecked(move |mut input| {
        let output = self.exec(input.reborrow());
        modifier(input);
        output
      })
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Default, PartialEq, Eq)]
  struct State {
    from: i32,
    to: i32,
  }

  fn accepter() -> C!((), State) {
    unsafe {
      wrap_unchecked(|input: Input<&mut State, &mut ()>| {
        input.state.to = input.state.from;
        input.digest(1)
      })
    }
  }

  fn rejecter() -> C!((), State) {
    unsafe {
      wrap_unchecked(|input: Input<&mut State, &mut ()>| {
        input.state.to = input.state.from;
        None
      })
    }
  }

  #[test]
  fn combinator_prepare() {
    let mut state = State::default();
    assert!(accepter()
      .prepare(|input| {
        input.state.from = 1;
      })
      .exec(Input::new("123", 0, &mut state, &mut ()).unwrap())
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
      .exec(Input::new("123", 0, &mut state, &mut ()).unwrap())
      .is_some());
    assert_eq!(state, State { from: 1, to: 0 });

    let mut state = State::default();
    assert!(rejecter()
      .then(|ctx| {
        ctx.input.state.from = 1;
      })
      .exec(Input::new("123", 0, &mut state, &mut ()).unwrap())
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
      .exec(Input::new("123", 0, &mut state, &mut ()).unwrap())
      .is_some());
    assert_eq!(state, State { from: 0, to: 0 });

    let mut state = State::default();
    assert!(rejecter()
      .catch(|input| {
        input.state.from = 1;
      })
      .exec(Input::new("123", 0, &mut state, &mut ()).unwrap())
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
      .exec(Input::new("123", 0, &mut state, &mut ()).unwrap())
      .is_some());
    assert_eq!(state, State { from: 0, to: 1 });

    let mut state = State::default();
    assert!(rejecter()
      .finally(|input| {
        input.state.to = 1;
      })
      .exec(Input::new("123", 0, &mut state, &mut ()).unwrap())
      .is_none());
    assert_eq!(state, State { from: 0, to: 1 });
  }
}
