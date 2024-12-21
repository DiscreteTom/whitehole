use super::AcceptedContext;
use crate::{
  combinator::{wrap, Action, Combinator, Input, Output},
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
  pub fn prepare(self, modifier: impl Fn(&mut Input<&mut T::State, &mut T::Heap>)) -> C!(@T) {
    wrap(move |input| {
      modifier(input);
      self.exec(input)
    })
  }

  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// after being accepted.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: C!((), MyState)) {
  /// combinator.then(|ctx| ctx.input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn then(
    self,
    modifier: impl for<'text> Fn(
      AcceptedContext<&mut Input<'text, &mut T::State, &mut T::Heap>, &Output<'text, T::Value>>,
    ),
  ) -> C!(@T) {
    wrap(move |input| {
      self.exec(input).inspect(|output| {
        modifier(AcceptedContext { input, output });
      })
    })
  }

  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// after being rejected.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: C!((), MyState)) {
  /// combinator.rollback(|input| input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn rollback(self, modifier: impl Fn(&mut Input<&mut T::State, &mut T::Heap>)) -> C!(@T) {
    wrap(move |input| {
      let output = self.exec(input);
      if output.is_none() {
        modifier(input);
      }
      output
    })
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
    wrap(|input: &mut Input<&mut State, &mut ()>| {
      input.state.to = input.state.from;
      Some(Output {
        value: (),
        rest: &input.rest()[1..],
      })
    })
  }

  fn rejecter() -> C!((), State) {
    wrap(|input: &mut Input<&mut State, &mut ()>| {
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
      .exec(&mut Input::new("123", 0, &mut state, &mut ()).unwrap())
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
      .exec(&mut Input::new("123", 0, &mut state, &mut ()).unwrap())
      .is_some());
    assert_eq!(state, State { from: 1, to: 0 });

    let mut state = State::default();
    assert!(rejecter()
      .then(|ctx| {
        ctx.input.state.from = 1;
      })
      .exec(&mut Input::new("123", 0, &mut state, &mut ()).unwrap())
      .is_none());
    assert_eq!(state, State { from: 0, to: 0 });
  }

  #[test]
  fn combinator_rollback() {
    let mut state = State::default();
    assert!(accepter()
      .rollback(|input| {
        input.state.from = 1;
      })
      .exec(&mut Input::new("123", 0, &mut state, &mut ()).unwrap())
      .is_some());
    assert_eq!(state, State { from: 0, to: 0 });

    let mut state = State::default();
    assert!(rejecter()
      .rollback(|input| {
        input.state.from = 1;
      })
      .exec(&mut Input::new("123", 0, &mut state, &mut ()).unwrap())
      .is_none());
    assert_eq!(state, State { from: 1, to: 0 });
  }
}
