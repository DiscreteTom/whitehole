use super::{create_closure_decorator, Accepted};
use crate::{
  action::Context,
  combinator::{Action, Combinator, Output},
  instant::Instant,
};

create_closure_decorator!(Prepare, "See [`Combinator::prepare`].");
create_closure_decorator!(Then, "See [`Combinator::then`].");
create_closure_decorator!(Catch, "See [`Combinator::catch`].");
create_closure_decorator!(Finally, "See [`Combinator::finally`].");

unsafe impl<Text: ?Sized, T: Action<Text>, D: Fn(&Instant<&Text>, Context<&mut T::State, &mut T::Heap>)>
  Action<Text> for Prepare<T, D>
{
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    mut ctx: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    (self.inner)(instant, ctx.reborrow());
    self.action.exec(instant, ctx)
  }
}

unsafe impl<
    Text: ?Sized,
    T: Action<Text>,
    D: Fn(Accepted<&Text, &T::Value>, Context<&mut T::State, &mut T::Heap>),
  > Action<Text> for Then<T, D>
{
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    mut ctx: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.action.exec(instant, ctx.reborrow()).inspect(|output| {
      (self.inner)(Accepted::new(instant, output.as_ref()), ctx);
    })
  }
}

unsafe impl<Text: ?Sized, T: Action<Text>, D: Fn(&Instant<&Text>, Context<&mut T::State, &mut T::Heap>)>
  Action<Text> for Catch<T, D>
{
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    mut ctx: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let output = self.action.exec(instant, ctx.reborrow());
    if output.is_none() {
      (self.inner)(instant, ctx);
    }
    output
  }
}

unsafe impl<Text: ?Sized, T: Action<Text>, D: Fn(&Instant<&Text>, Context<&mut T::State, &mut T::Heap>)>
  Action<Text> for Finally<T, D>
{
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    mut ctx: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let output = self.action.exec(instant, ctx.reborrow());
    (self.inner)(instant, ctx);
    output
  }
}

impl<T> Combinator<T> {
  /// Create a new combinator to modify [`Context::state`] and [`Context::heap`]
  /// before being executed.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: Combinator<impl Action<str, MyState>>) {
  /// combinator.prepare(|input, ctx| ctx.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn prepare<Text: ?Sized, F: Fn(&Instant<&Text>, Context<&mut T::State, &mut T::Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Prepare<T, F>>
  where
    T: Action<Text>,
  {
    Combinator::new(Prepare::new(self.action, modifier))
  }

  /// Create a new combinator to modify [`Context::state`] and [`Context::heap`]
  /// after being accepted.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: Combinator<impl Action<str, MyState>>) {
  /// combinator.then(|_, mut ctx| ctx.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn then<
    Text: ?Sized,
    F: Fn(Accepted<&Text, &T::Value>, Context<&mut T::State, &mut T::Heap>),
  >(
    self,
    modifier: F,
  ) -> Combinator<Then<T, F>>
  where
    T: Action<Text>,
  {
    Combinator::new(Then::new(self.action, modifier))
  }

  /// Create a new combinator to modify [`Context::state`] and [`Context::heap`]
  /// after being rejected.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: Combinator<impl Action<str, MyState>>) {
  /// combinator.catch(|_, ctx| ctx.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn catch<Text: ?Sized, F: Fn(&Instant<&Text>, Context<&mut T::State, &mut T::Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Catch<T, F>>
  where
    T: Action<Text>,
  {
    Combinator::new(Catch::new(self.action, modifier))
  }

  /// Create a new combinator to modify [`Context::state`] and [`Context::heap`]
  /// after the combinator is executed,
  /// no matter whether it is accepted or rejected.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { value: i32 }
  /// # fn t(combinator: Combinator<impl Action<str, MyState>>) {
  /// combinator.finally(|_, ctx| ctx.state.value += 1)
  /// # ;}
  /// ```
  #[inline]
  pub fn finally<Text: ?Sized, F: Fn(&Instant<&Text>, Context<&mut T::State, &mut T::Heap>)>(
    self,
    modifier: F,
  ) -> Combinator<Finally<T, F>>
  where
    T: Action<Text>,
  {
    Combinator::new(Finally::new(self.action, modifier))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{contextual, digest::Digest, instant::Instant};
  use std::{ops::RangeFrom, slice::SliceIndex};

  #[derive(Debug, Default, PartialEq, Eq)]
  struct State {
    from: i32,
    to: i32,
  }

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text, State = State, Heap = (), Value = ()>,
    input: &Text,
    state: &mut State,
    digested: Option<usize>,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action
        .exec(
          &Instant::new(input),
          Context {
            state,
            heap: &mut ()
          }
        )
        .map(|o| o.digested),
      digested
    )
  }

  contextual!(State, ());

  fn accepter() -> Combinator<impl Action<str, State = State, Heap = (), Value = ()>> {
    wrap(|instant| instant.accept(1)).prepare(|_, ctx| ctx.state.to = ctx.state.from)
  }
  fn accepter_bytes() -> Combinator<impl Action<[u8], State = State, Heap = (), Value = ()>> {
    bytes::wrap(|instant| instant.accept(1)).prepare(|_, ctx| ctx.state.to = ctx.state.from)
  }

  fn rejecter() -> Combinator<impl Action<str, State = State, Heap = (), Value = ()>> {
    wrap(|_| None).prepare(|_, ctx| ctx.state.to = ctx.state.from)
  }
  fn rejecter_bytes() -> Combinator<impl Action<[u8], State = State, Heap = (), Value = ()>> {
    bytes::wrap(|_| None).prepare(|_, ctx| ctx.state.to = ctx.state.from)
  }

  #[test]
  fn combinator_prepare() {
    // accepted
    let mut state = State::default();
    helper(
      accepter().prepare(|_, ctx| {
        ctx.state.from = 1;
      }),
      "123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 1, to: 1 });
    let mut state = State::default();
    helper(
      accepter_bytes().prepare(|_, ctx| {
        ctx.state.from = 1;
      }),
      b"123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 1, to: 1 });

    // rejected
    let mut state = State::default();
    helper(
      rejecter().prepare(|_, ctx| {
        ctx.state.from = 1;
      }),
      "123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 1, to: 1 });
    let mut state = State::default();
    helper(
      rejecter_bytes().prepare(|_, ctx| {
        ctx.state.from = 1;
      }),
      b"123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 1, to: 1 });
  }

  #[test]
  fn combinator_then() {
    // accepted
    let mut state = State::default();
    helper(
      accepter().then(|_, ctx| {
        ctx.state.from = 1;
      }),
      "123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 1, to: 0 });
    let mut state = State::default();
    helper(
      accepter_bytes().then(|_, ctx| {
        ctx.state.from = 1;
      }),
      b"123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 1, to: 0 });

    // rejected
    let mut state = State::default();
    helper(
      rejecter().then(|_, ctx| {
        ctx.state.from = 1;
      }),
      "123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 0, to: 0 });
    let mut state = State::default();
    helper(
      rejecter_bytes().then(|_, ctx| {
        ctx.state.from = 1;
      }),
      b"123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 0, to: 0 });
  }

  #[test]
  fn combinator_catch() {
    // accepted
    let mut state = State::default();
    helper(
      accepter().catch(|_, ctx| {
        ctx.state.from = 1;
      }),
      "123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 0, to: 0 });
    let mut state = State::default();
    helper(
      accepter_bytes().catch(|_, ctx| {
        ctx.state.from = 1;
      }),
      b"123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 0, to: 0 });

    // rejected
    let mut state = State::default();
    helper(
      rejecter().catch(|_, ctx| {
        ctx.state.from = 1;
      }),
      "123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 1, to: 0 });
    let mut state = State::default();
    helper(
      rejecter_bytes().catch(|_, ctx| {
        ctx.state.from = 1;
      }),
      b"123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 1, to: 0 });
  }

  #[test]
  fn combinator_finally() {
    // accepted
    let mut state = State::default();
    helper(
      accepter().finally(|_, ctx| {
        ctx.state.to = 1;
      }),
      "123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 0, to: 1 });
    let mut state = State::default();
    helper(
      accepter_bytes().finally(|_, ctx| {
        ctx.state.to = 1;
      }),
      b"123",
      &mut state,
      Some(1),
    );
    assert_eq!(state, State { from: 0, to: 1 });

    // rejected
    let mut state = State::default();
    helper(
      rejecter().finally(|_, ctx| {
        ctx.state.to = 1;
      }),
      "123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 0, to: 1 });
    let mut state = State::default();
    helper(
      rejecter_bytes().finally(|_, ctx| {
        ctx.state.to = 1;
      }),
      b"123",
      &mut state,
      None,
    );
    assert_eq!(state, State { from: 0, to: 1 });
  }
}
