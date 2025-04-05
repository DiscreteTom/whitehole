//! Decorators that modify the acceptance of a combinator.

use super::{create_closure_decorator, create_simple_decorator, Accepted};
use crate::{
  action::Input,
  combinator::{Action, Combinator, Output},
  digest::Digest,
  instant::Instant,
};

create_closure_decorator!(When, "See [`Combinator::when`].");
create_closure_decorator!(Prevent, "See [`Combinator::prevent`].");
create_closure_decorator!(Reject, "See [`Combinator::reject`].");
create_simple_decorator!(Optional, "See [`Combinator::optional`].");
create_simple_decorator!(Boundary, "See [`Combinator::boundary`].");

unsafe impl<T: Action, D: Fn(Input<&Instant<&T::Text>, &mut T::State, &mut T::Heap>) -> bool> Action
  for When<T, D>
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
    if (self.inner)(input.reborrow()) {
      self.action.exec(input)
    } else {
      None
    }
  }
}

unsafe impl<T: Action, D: Fn(Input<&Instant<&T::Text>, &mut T::State, &mut T::Heap>) -> bool> Action
  for Prevent<T, D>
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
    if !(self.inner)(input.reborrow()) {
      self.action.exec(input)
    } else {
      None
    }
  }
}

unsafe impl<
    T: Action<Text: Digest>,
    D: Fn(Accepted<&Instant<&T::Text>, &mut T::State, &mut T::Heap, &T::Value>) -> bool,
  > Action for Reject<T, D>
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
    self.action.exec(input.reborrow()).and_then(|output| {
      if (self.inner)(unsafe {
        Accepted::new_unchecked(input.instant, output.as_ref(), input.state, input.heap)
      }) {
        None
      } else {
        output.into()
      }
    })
  }
}

unsafe impl<T: Action<Value: Default>> Action for Optional<T> {
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    Some(self.action.exec(input).unwrap_or_else(|| Output {
      value: Default::default(),
      digested: 0,
    }))
  }
}

unsafe impl<T: Action<Text = str>> Action for Boundary<T> {
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let rest = input.instant.rest();
    self.action.exec(input).and_then(|output| {
      unsafe { rest.get_unchecked(output.digested..) }
        .chars()
        .next()
        .is_none_or(|c| !c.is_alphanumeric() && c != '_')
        .then_some(output)
    })
  }
}

impl<T> Combinator<T> {
  /// Create a new combinator to check the [`Input`] before being executed.
  /// The combinator will be executed only if the `condition` returns `true`.
  ///
  /// This is the opposite of [`Combinator::prevent`].
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { execute: bool }
  /// # fn t(combinator: Combinator<impl Action<Text=str, State=MyState>>) {
  /// combinator.when(|input| input.state.execute)
  /// # ;}
  /// ```
  #[inline]
  pub fn when<F: Fn(Input<&Instant<&T::Text>, &mut T::State, &mut T::Heap>) -> bool>(
    self,
    condition: F,
  ) -> Combinator<When<T, F>>
  where
    T: Action,
  {
    Combinator::new(When::new(self.action, condition))
  }

  /// Create a new combinator to check the [`Input`] before being executed.
  /// The combinator will reject if the `preventer` returns `true`.
  ///
  /// This is the opposite of [`Combinator::when`].
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { reject: bool }
  /// # fn t(combinator: Combinator<impl Action<Text=str, State=MyState>>) {
  /// combinator.prevent(|input| input.state.reject)
  /// # ;}
  /// ```
  #[inline]
  pub fn prevent<F: Fn(Input<&Instant<&T::Text>, &mut T::State, &mut T::Heap>) -> bool>(
    self,
    preventer: F,
  ) -> Combinator<Prevent<T, F>>
  where
    T: Action,
  {
    Combinator::new(Prevent::new(self.action, preventer))
  }

  /// Create a new combinator to check the [`Accepted`] after being executed.
  /// The combinator will reject if the `rejecter` returns `true`.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, digest::Digest, combinator::Combinator};
  /// # fn t(combinator: Combinator<impl Action<Text=str>>) {
  /// combinator.reject(|accepted| accepted.content() != "123")
  /// # ;}
  /// ```
  #[inline]
  pub fn reject<
    F: Fn(Accepted<&Instant<&T::Text>, &mut T::State, &mut T::Heap, &T::Value>) -> bool,
  >(
    self,
    rejecter: F,
  ) -> Combinator<Reject<T, F>>
  where
    T: Action,
  {
    Combinator::new(Reject::new(self.action, rejecter))
  }

  /// Make the combinator optional.
  ///
  /// Under the hood, if the original combinator rejects, the new combinator will accept
  /// with the default value and zero digested.
  ///
  /// This requires the `Value` to implement [`Default`],
  /// thus usually used before setting a custom value.
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # #[derive(Clone)]
  /// # struct MyValue;
  /// # fn t(combinator: Combinator<impl Action>) {
  /// // make the combinator optional before binding a value
  /// combinator.optional().bind(MyValue)
  /// // instead of
  /// // combinator.bind(MyValue).optional()
  /// # ;}
  /// ```
  /// Or you can wrap `Value` with [`Option`] to make it optional
  /// after setting a custom value.
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # #[derive(Clone)]
  /// # struct MyValue;
  /// # fn t(combinator: Combinator<impl Action>) {
  /// combinator.bind(Some(MyValue)).optional()
  /// # ;}
  /// ```
  /// # Caveats
  /// Be careful of infinite loops since this may accept with 0 bytes digested.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # fn t(combinator: Combinator<impl Action>) {
  /// combinator.optional()
  /// # ;}
  /// ```
  #[inline]
  pub fn optional(self) -> Combinator<Optional<T>> {
    Combinator::new(Optional::new(self.action))
  }

  /// Create a new combinator to reject after execution
  /// if the next undigested char is alphanumeric or `_`.
  /// See [`char::is_alphanumeric`].
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # fn t(combinator: Combinator<impl Action>) {
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
  use crate::{contextual, digest::Digest, instant::Instant};
  use std::{fmt::Debug, ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text = Text, State = bool, Heap = (), Value = ()>,
    input: &Text,
    state: &mut bool,
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

  contextual!(bool, ());

  fn accepter(
  ) -> Combinator<impl Action<Text = str, State = bool, Heap = (), Value = ()> + Debug + Copy> {
    wrap(|input| {
      *input.state = true;
      input.instant.accept(1)
    })
  }
  fn accepter_bytes(
  ) -> Combinator<impl Action<Text = [u8], State = bool, Heap = (), Value = ()> + Debug + Copy> {
    bytes::wrap(|input| {
      *input.state = true;
      input.instant.accept(1)
    })
  }

  fn rejecter(
  ) -> Combinator<impl Action<Text = str, State = bool, Heap = (), Value = ()> + Debug + Copy> {
    wrap(|input| {
      *input.state = true;
      None
    })
  }
  fn rejecter_bytes(
  ) -> Combinator<impl Action<Text = [u8], State = bool, Heap = (), Value = ()> + Debug + Copy> {
    bytes::wrap(|input| {
      *input.state = true;
      None
    })
  }

  #[test]
  fn combinator_when() {
    // prevented
    let mut executed = false;
    helper(accepter().when(|_| false), "123", &mut executed, None);
    assert!(!executed);
    let mut executed = false;
    helper(
      accepter_bytes().when(|_| false),
      b"123",
      &mut executed,
      None,
    );
    assert!(!executed);

    // executed
    let mut executed = false;
    helper(accepter().when(|_| true), "123", &mut executed, Some(1));
    assert!(executed);
    let mut executed = false;
    helper(
      accepter_bytes().when(|_| true),
      b"123",
      &mut executed,
      Some(1),
    );
    assert!(executed);

    // debug
    let _ = format!("{:?}", accepter().when(|_| true));
    // copy & clone
    let c = accepter().when(|_| true);
    let _c = c;
    let _c = c.clone();
  }

  #[test]
  fn combinator_prevent() {
    // prevented
    let mut executed = false;
    helper(accepter().prevent(|_| true), "123", &mut executed, None);
    assert!(!executed);
    let mut executed = false;
    helper(
      accepter_bytes().prevent(|_| true),
      b"123",
      &mut executed,
      None,
    );
    assert!(!executed);

    // executed
    let mut executed = false;
    helper(accepter().prevent(|_| false), "123", &mut executed, Some(1));
    assert!(executed);
    let mut executed = false;
    helper(
      accepter_bytes().prevent(|_| false),
      b"123",
      &mut executed,
      Some(1),
    );
    assert!(executed);

    // debug
    let _ = format!("{:?}", accepter().prevent(|_| true));
    // copy & clone
    let c = accepter().prevent(|_| true);
    let _c = c;
    let _c = c.clone();
  }

  #[test]
  fn combinator_reject() {
    // accepted
    let mut executed = false;
    helper(
      accepter().reject(|accept| accept.content() != "1"),
      "123",
      &mut executed,
      Some(1),
    );
    assert!(executed);
    let mut executed = false;
    helper(
      accepter_bytes().reject(|accept| accept.content() != b"1"),
      b"123",
      &mut executed,
      Some(1),
    );
    assert!(executed);

    // rejected
    let mut executed = false;
    helper(
      accepter().reject(|accept| accept.content() == "1"),
      "123",
      &mut executed,
      None,
    );
    assert!(executed);
    let mut executed = false;
    helper(
      accepter_bytes().reject(|accept| accept.content() == b"1"),
      b"123",
      &mut executed,
      None,
    );
    assert!(executed);

    // debug
    let _ = format!("{:?}", accepter().reject(|accept| accept.content() != "1"));
    // copy & clone
    let c = accepter().reject(|accept| accept.content() != "1");
    let _c = c;
    let _c = c.clone();
  }

  #[test]
  fn combinator_optional() {
    // accept
    let mut executed = false;
    helper(accepter().optional(), "123", &mut executed, Some(1));
    assert!(executed);
    let mut executed = false;
    helper(accepter_bytes().optional(), b"123", &mut executed, Some(1));
    assert!(executed);

    // reject but optional
    let mut executed = false;
    helper(rejecter().optional(), "123", &mut executed, Some(0));
    assert!(executed);
    let mut executed = false;
    helper(rejecter_bytes().optional(), b"123", &mut executed, Some(0));
    assert!(executed);

    // debug
    let _ = format!("{:?}", accepter().optional());
    // copy & clone
    let c = accepter().optional();
    let _c = c;
    let _c = c.clone();
  }

  #[test]
  fn optional_can_be_the_last_one() {
    let mut executed = false;
    helper(accepter().optional(), "", &mut executed, Some(0));
    assert!(executed);
    let mut executed = false;
    helper(accepter_bytes().optional(), b"", &mut executed, Some(0));
    assert!(executed);
  }

  #[test]
  fn combinator_boundary() {
    let mut executed = false;
    helper(accepter().boundary(), "1", &mut executed, Some(1));
    assert!(executed);

    let mut executed = false;
    helper(accepter().boundary(), "1.", &mut executed, Some(1));
    assert!(executed);

    let mut executed = false;
    helper(accepter().boundary(), "12", &mut executed, None);
    assert!(executed);

    let mut executed = false;
    helper(accepter().boundary(), "1a", &mut executed, None);
    assert!(executed);

    let mut executed = false;
    helper(accepter().boundary(), "1_", &mut executed, None);
    assert!(executed);

    let mut executed = false;
    helper(accepter().boundary(), "1好", &mut executed, None);
    assert!(executed);

    // debug
    let _ = format!("{:?}", accepter().boundary());
    // copy & clone
    let c = accepter().boundary();
    let _c = c;
    let _c = c.clone();
  }
}
