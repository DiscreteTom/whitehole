use super::AcceptedOutputContext;
use crate::combinator::{Combinator, Input, Output};

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Combinator<'a, Kind, State, Heap> {
  /// Modify [`Input::state`] and [`Input::heap`] before the combinator is executed.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.prepare(|input| input.state.value += 1)
  /// # ;}
  /// ```
  pub fn prepare(self, modifier: impl Fn(&mut Input<&mut State, &mut Heap>) + 'a) -> Self {
    Combinator::boxed(move |input| {
      modifier(input);
      self.parse(input)
    })
  }

  /// Modify [`Input::state`] and [`Input::heap`] if the combinator is accepted.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.then(|ctx| ctx.input.state.value += 1)
  /// # ;}
  /// ```
  pub fn then(
    self,
    modifier: impl for<'text> Fn(
        AcceptedOutputContext<&mut Input<'text, &mut State, &mut Heap>, &Output<'text, Kind>>,
      ) + 'a,
  ) -> Self {
    Combinator::boxed(move |input| {
      self.parse(input).inspect(|output| {
        modifier(AcceptedOutputContext { input, output });
      })
    })
  }

  /// Modify [`Input::state`] and [`Input::heap`] if the combinator is rejected.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.rollback(|input| input.state.value += 1)
  /// # ;}
  /// ```
  pub fn rollback(self, modifier: impl Fn(&mut Input<&mut State, &mut Heap>) + 'a) -> Self {
    Combinator::boxed(move |input| {
      let output = self.parse(input);
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

  fn accepter() -> Combinator<'static, (), State, ()> {
    Combinator::boxed(|input: &mut Input<&mut State, &mut ()>| {
      input.state.to = input.state.from;
      Some(Output {
        kind: (),
        rest: &input.rest()[1..],
      })
    })
  }

  fn rejecter() -> Combinator<'static, (), State, ()> {
    Combinator::boxed(|input: &mut Input<&mut State, &mut ()>| {
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
      .parse(&mut Input::new("123", 0, &mut state, &mut ()).unwrap())
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
      .parse(&mut Input::new("123", 0, &mut state, &mut ()).unwrap())
      .is_some());
    assert_eq!(state, State { from: 1, to: 0 });

    let mut state = State::default();
    assert!(rejecter()
      .then(|ctx| {
        ctx.input.state.from = 1;
      })
      .parse(&mut Input::new("123", 0, &mut state, &mut ()).unwrap())
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
      .parse(&mut Input::new("123", 0, &mut state, &mut ()).unwrap())
      .is_some());
    assert_eq!(state, State { from: 0, to: 0 });

    let mut state = State::default();
    assert!(rejecter()
      .rollback(|input| {
        input.state.from = 1;
      })
      .parse(&mut Input::new("123", 0, &mut state, &mut ()).unwrap())
      .is_none());
    assert_eq!(state, State { from: 1, to: 0 });
  }
}
