use super::{create_closure_decorator, Accepted};
use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
  digest::Digest,
  instant::Instant,
};

create_closure_decorator!(Prepare, "See [`Combinator::prepare`].");
create_closure_decorator!(Then, "See [`Combinator::then`].");
create_closure_decorator!(Catch, "See [`Combinator::catch`].");
create_closure_decorator!(Finally, "See [`Combinator::finally`].");

unsafe impl<T: Action, D: Fn(Input<&Instant<&T::Text>, &mut T::State, &mut T::Heap>)> Action
  for Prepare<T, D>
{
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    (self.inner)(input.reborrow());
    self.action.exec(input)
  }
}

unsafe impl<
    T: Action<Text: Digest>,
    D: Fn(Accepted<&Instant<&T::Text>, &mut T::State, &mut T::Heap, &T::Value>),
  > Action for Then<T, D>
{
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.action.exec(input.reborrow()).inspect(|output| {
      (self.inner)(unsafe {
        Accepted::new_unchecked(input.instant, output.as_ref(), input.state, input.heap)
      });
    })
  }
}

unsafe impl<T: Action, D: Fn(Input<&Instant<&T::Text>, &mut T::State, &mut T::Heap>)> Action
  for Catch<T, D>
{
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let output = self.action.exec(input.reborrow());
    if output.is_none() {
      (self.inner)(input);
    }
    output
  }
}

unsafe impl<T: Action, D: Fn(Input<&Instant<&T::Text>, &mut T::State, &mut T::Heap>)> Action
  for Finally<T, D>
{
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
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
  /// # fn t(combinator: Combinator<impl Action<Text=str, State=MyState>>) {
  /// combinator.prepare(|input| input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn prepare<F: Fn(Input<&Instant<&T::Text>, &mut T::State, &mut T::Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Prepare<T, F>>
  where
    T: Action,
  {
    Combinator::new(Prepare::new(self.action, modifier))
  }

  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// after being accepted.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: Combinator<impl Action<Text=str, State=MyState>>) {
  /// combinator.then(|input| input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn then<F: Fn(Accepted<&Instant<&T::Text>, &mut T::State, &mut T::Heap, &T::Value>)>(
    self,
    modifier: F,
  ) -> Combinator<Then<T, F>>
  where
    T: Action,
  {
    Combinator::new(Then::new(self.action, modifier))
  }

  /// Create a new combinator to modify [`Input::state`] and [`Input::heap`]
  /// after being rejected.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: Combinator<impl Action<Text=str, State=MyState>>) {
  /// combinator.catch(|input| input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn catch<F: Fn(Input<&Instant<&T::Text>, &mut T::State, &mut T::Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Catch<T, F>>
  where
    T: Action,
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
  /// # fn t(combinator: Combinator<impl Action<Text=str, State=MyState>>) {
  /// combinator.finally(|input| input.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn finally<F: Fn(Input<&Instant<&T::Text>, &mut T::State, &mut T::Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Finally<T, F>>
  where
    T: Action,
  {
    Combinator::new(Finally::new(self.action, modifier))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{contextual, digest::Digest, instant::Instant};
  use std::{fmt::Debug, ops::RangeFrom, slice::SliceIndex};

  #[derive(Debug, Default, PartialEq, Eq)]
  pub struct State {
    from: i32,
    to: i32,
  }

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text = Text, State = State, Heap = (), Value = ()>,
    input: &Text,
    state: &mut State,
    digested: Option<usize>,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action
        .exec(Input {
          instant: &Instant::new(input),
          state,
          heap: &mut ()
        })
        .map(|o| o.digested),
      digested
    )
  }

  contextual!(State, ());

  fn accepter(
  ) -> Combinator<impl Action<Text = str, State = State, Heap = (), Value = ()> + Debug + Copy> {
    wrap(|input| {
      input.state.to = input.state.from;
      input.instant.accept(1)
    })
  }
  fn accepter_bytes(
  ) -> Combinator<impl Action<Text = [u8], State = State, Heap = (), Value = ()> + Debug + Copy> {
    bytes::wrap(|input| {
      input.state.to = input.state.from;
      input.instant.accept(1)
    })
  }

  fn rejecter(
  ) -> Combinator<impl Action<Text = str, State = State, Heap = (), Value = ()> + Debug + Copy> {
    wrap(|input| {
      input.state.to = input.state.from;
      None
    })
  }
  fn rejecter_bytes(
  ) -> Combinator<impl Action<Text = [u8], State = State, Heap = (), Value = ()> + Debug + Copy> {
    bytes::wrap(|input| {
      input.state.to = input.state.from;
      None
    })
  }

  #[test]
  fn combinator_prepare() {
    // accepted
    let mut state = State::default();
    helper(
      accepter().prepare(|input| {
        input.state.from = 1;
      }),
      "123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 1, to: 1 });
    let mut state = State::default();
    helper(
      accepter_bytes().prepare(|input| {
        input.state.from = 1;
      }),
      b"123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 1, to: 1 });

    // rejected
    let mut state = State::default();
    helper(
      rejecter().prepare(|input| {
        input.state.from = 1;
      }),
      "123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 1, to: 1 });
    let mut state = State::default();
    helper(
      rejecter_bytes().prepare(|input| {
        input.state.from = 1;
      }),
      b"123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 1, to: 1 });

    // debug
    let _ = format!("{:?}", accepter().prepare(|_| {}));
    // copy & clone
    let c = accepter().prepare(|_| {});
    let _c = c;
    let _c = c.clone();
  }

  #[test]
  fn combinator_then() {
    // accepted
    let mut state = State::default();
    helper(
      accepter().then(|input| {
        input.state.from = 1;
      }),
      "123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 1, to: 0 });
    let mut state = State::default();
    helper(
      accepter_bytes().then(|input| {
        input.state.from = 1;
      }),
      b"123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 1, to: 0 });

    // rejected
    let mut state = State::default();
    helper(
      rejecter().then(|input| {
        input.state.from = 1;
      }),
      "123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 0, to: 0 });
    let mut state = State::default();
    helper(
      rejecter_bytes().then(|input| {
        input.state.from = 1;
      }),
      b"123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 0, to: 0 });

    // debug
    let _ = format!("{:?}", accepter().then(|_| {}));
    // copy & clone
    let c = accepter().then(|_| {});
    let _c = c;
    let _c = c.clone();
  }

  #[test]
  fn combinator_catch() {
    // accepted
    let mut state = State::default();
    helper(
      accepter().catch(|input| {
        input.state.from = 1;
      }),
      "123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 0, to: 0 });
    let mut state = State::default();
    helper(
      accepter_bytes().catch(|input| {
        input.state.from = 1;
      }),
      b"123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 0, to: 0 });

    // rejected
    let mut state = State::default();
    helper(
      rejecter().catch(|input| {
        input.state.from = 1;
      }),
      "123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 1, to: 0 });
    let mut state = State::default();
    helper(
      rejecter_bytes().catch(|input| {
        input.state.from = 1;
      }),
      b"123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 1, to: 0 });

    // debug
    let _ = format!("{:?}", accepter().catch(|_| {}));
    // copy & clone
    let c = accepter().catch(|_| {});
    let _c = c;
    let _c = c.clone();
  }

  #[test]
  fn combinator_finally() {
    // accepted
    let mut state = State::default();
    helper(
      accepter().finally(|input| {
        input.state.to = 1;
      }),
      "123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 0, to: 1 });
    let mut state = State::default();
    helper(
      accepter_bytes().finally(|input| {
        input.state.to = 1;
      }),
      b"123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 0, to: 1 });

    // rejected
    let mut state = State::default();
    helper(
      rejecter().finally(|input| {
        input.state.to = 1;
      }),
      "123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 0, to: 1 });
    let mut state = State::default();
    helper(
      rejecter_bytes().finally(|input| {
        input.state.to = 1;
      }),
      b"123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 0, to: 1 });

    // debug
    let _ = format!("{:?}", accepter().finally(|_| {}));
    // copy & clone
    let c = accepter().finally(|_| {});
    let _c = c;
    let _c = c.clone();
  }
}
