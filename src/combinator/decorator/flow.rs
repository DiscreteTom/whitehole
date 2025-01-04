//! Decorators that modify the acceptance of a combinator.

use super::{create_closure_decorator, create_simple_decorator, AcceptedContext};
use crate::combinator::{Action, Combinator, Input, Output};

create_closure_decorator!(When, "See [`Combinator::when`].");
create_closure_decorator!(Prevent, "See [`Combinator::prevent`].");
create_closure_decorator!(Reject, "See [`Combinator::reject`].");
create_simple_decorator!(Optional, "See [`Combinator::optional`].");
create_simple_decorator!(Boundary, "See [`Combinator::boundary`].");

unsafe impl<T: Action, D: Fn(Input<&mut T::State, &mut T::Heap>) -> bool> Action for When<T, D> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    if (self.inner)(input.reborrow()) {
      self.action.exec(input)
    } else {
      None
    }
  }
}

unsafe impl<T: Action, D: Fn(Input<&mut T::State, &mut T::Heap>) -> bool> Action for Prevent<T, D> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    if !(self.inner)(input.reborrow()) {
      self.action.exec(input)
    } else {
      None
    }
  }
}

unsafe impl<
    T: Action,
    D: Fn(AcceptedContext<Input<&mut T::State, &mut T::Heap>, &Output<T::Value>>) -> bool,
  > Action for Reject<T, D>
{
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.action.exec(input.reborrow()).and_then(|output| {
      if (self.inner)(AcceptedContext {
        input,
        output: &output,
      }) {
        None
      } else {
        output.into()
      }
    })
  }
}

unsafe impl<T: Action<Value: Default>> Action for Optional<T> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(&self, input: Input<&mut Self::State, &mut Self::Heap>) -> Option<Output<Self::Value>> {
    Some(self.action.exec(input).unwrap_or_else(|| Output {
      value: Default::default(),
      digested: 0,
    }))
  }
}

unsafe impl<T: Action> Action for Boundary<T> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    mut input: Input<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.action.exec(input.reborrow()).and_then(|output| {
      unsafe { input.instant().rest().get_unchecked(output.digested..) }
        .chars()
        .next()
        .map_or(true, |c| !c.is_alphanumeric() && c != '_')
        .then_some(output)
    })
  }
}

impl<T: Action> Combinator<T> {
  /// Create a new combinator to check the [`Input`] before being executed.
  /// The combinator will be executed only if the `condition` returns `true`.
  ///
  /// This is the opposite of [`Combinator::prevent`].
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # struct MyState { execute: bool }
  /// # fn t(combinator: C!((), MyState)) {
  /// combinator.when(|input| input.state.execute)
  /// # ;}
  /// ```
  #[inline]
  pub fn when<F: Fn(Input<&mut T::State, &mut T::Heap>) -> bool>(
    self,
    condition: F,
  ) -> Combinator<When<T, F>> {
    Combinator::new(When::new(self.action, condition))
  }

  /// Create a new combinator to check the [`Input`] before being executed.
  /// The combinator will reject if the `preventer` returns `true`.
  ///
  /// This is the opposite of [`Combinator::when`].
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # struct MyState { reject: bool }
  /// # fn t(combinator: C!((), MyState)) {
  /// combinator.prevent(|input| input.state.reject)
  /// # ;}
  /// ```
  #[inline]
  pub fn prevent<F: Fn(Input<&mut T::State, &mut T::Heap>) -> bool>(
    self,
    preventer: F,
  ) -> Combinator<Prevent<T, F>> {
    Combinator::new(Prevent::new(self.action, preventer))
  }

  /// Create a new combinator to check the [`Input`] and [`Output`] after being executed.
  /// The combinator will reject if the `rejecter` returns `true`.
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # fn t(combinator: C!()) {
  /// combinator.reject(|ctx| ctx.content() != "123")
  /// # ;}
  /// ```
  #[inline]
  pub fn reject<
    F: Fn(AcceptedContext<Input<&mut T::State, &mut T::Heap>, &Output<T::Value>>) -> bool,
  >(
    self,
    rejecter: F,
  ) -> Combinator<Reject<T, F>> {
    Combinator::new(Reject::new(self.action, rejecter))
  }

  /// Make the combinator optional.
  ///
  /// Under the hood, the combinator will be accepted
  /// with the default value and zero digested if the original combinator rejects.
  /// # Caveats
  /// This requires the `Value` to implement [`Default`],
  /// thus usually used before setting a custom value.
  /// ```
  /// # use whitehole::C;
  /// # #[derive(Clone)]
  /// # struct MyValue;
  /// # fn t(combinator: C!()) {
  /// // make the combinator optional before binding a value
  /// combinator.optional().bind(MyValue)
  /// // instead of
  /// // combinator.bind(MyValue).optional()
  /// # ;}
  /// ```
  /// Or you can wrap `Value` with [`Option`] to make it optional
  /// after setting a custom value.
  /// ```
  /// # use whitehole::C;
  /// # #[derive(Clone)]
  /// # struct MyValue;
  /// # fn t(combinator: C!()) {
  /// combinator.bind(Some(MyValue)).optional()
  /// # ;}
  /// ```
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # fn t(combinator: C!()) {
  /// combinator.optional()
  /// # ;}
  /// ```
  #[inline]
  pub fn optional(self) -> Combinator<Optional<T>>
  where
    T::Value: Default,
  {
    Combinator::new(Optional::new(self.action))
  }

  /// Create a new combinator to reject after execution
  /// if the next undigested char is alphanumeric or `_`.
  /// See [`char::is_alphanumeric`].
  /// # Examples
  /// ```
  /// # use whitehole::C;
  /// # fn t(combinator: C!()) {
  /// combinator.boundary()
  /// # ;}
  /// ```
  #[inline]
  pub fn boundary(self) -> Combinator<Boundary<T>> {
    Combinator::new(Boundary::new(self.action))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::wrap, instant::Instant, C};

  fn accepter() -> C!((), bool) {
    wrap(|input| {
      *input.state = true;
      input.digest(1)
    })
  }

  fn rejecter() -> C!((), bool) {
    wrap(|input| {
      *input.state = true;
      None
    })
  }

  #[test]
  fn combinator_when() {
    let mut executed = false;
    assert!(accepter()
      .when(|_| false)
      .exec(Input::new(Instant::new("123"), &mut executed, &mut ()).unwrap())
      .is_none());
    assert!(!executed);

    let mut executed = false;
    assert!(accepter()
      .when(|_| true)
      .exec(Input::new(Instant::new("123"), &mut executed, &mut ()).unwrap())
      .is_some());
    assert!(executed);
  }

  #[test]
  fn combinator_prevent() {
    let mut executed = false;
    assert!(accepter()
      .prevent(|_| true)
      .exec(Input::new(Instant::new("123"), &mut executed, &mut ()).unwrap())
      .is_none());
    assert!(!executed);

    let mut executed = false;
    assert!(accepter()
      .prevent(|_| false)
      .exec(Input::new(Instant::new("123"), &mut executed, &mut ()).unwrap())
      .is_some());
    assert!(executed);
  }

  #[test]
  fn combinator_reject() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .reject(|input| input.content() != "1")
        .exec(Input::new(Instant::new("123"), &mut executed, &mut ()).unwrap())
        .unwrap()
        .digested,
      1
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .reject(|input| input.content() == "1")
        .exec(Input::new(Instant::new("123"), &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);
  }

  #[test]
  fn combinator_optional() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .optional()
        .exec(Input::new(Instant::new("123"), &mut executed, &mut ()).unwrap())
        .unwrap()
        .digested,
      1
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      rejecter()
        .optional()
        .exec(Input::new(Instant::new("123"), &mut executed, &mut ()).unwrap())
        .unwrap()
        .digested,
      0
    );
    assert!(executed);
  }

  #[test]
  fn combinator_boundary() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .boundary()
        .exec(Input::new(Instant::new("1"), &mut executed, &mut ()).unwrap())
        .unwrap()
        .digested,
      1
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .boundary()
        .exec(Input::new(Instant::new("12"), &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .boundary()
        .exec(Input::new(Instant::new("1a"), &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .boundary()
        .exec(Input::new(Instant::new("1_"), &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);
  }
}
