//! Decorators that modify the acceptance of a combinator.

use super::{create_closure_decorator, create_simple_decorator, Accepted};
use crate::{
  action::Context,
  combinator::{Action, Combinator, Output},
  instant::Instant,
};

create_closure_decorator!(When, "See [`Combinator::when`].");
create_closure_decorator!(Prevent, "See [`Combinator::prevent`].");
create_closure_decorator!(Reject, "See [`Combinator::reject`].");
create_simple_decorator!(Optional, "See [`Combinator::optional`].");
create_simple_decorator!(Boundary, "See [`Combinator::boundary`].");

unsafe impl<
    Text: ?Sized,
    State,
    Heap,
    T: Action<Text, State, Heap>,
    D: Fn(&Instant<&Text>, Context<&mut State, &mut Heap>) -> bool,
  > Action<Text, State, Heap> for When<T, D>
{
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    mut ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    if (self.inner)(instant, ctx.reborrow()) {
      self.action.exec(instant, ctx)
    } else {
      None
    }
  }
}

unsafe impl<
    Text: ?Sized,
    State,
    Heap,
    T: Action<Text, State, Heap>,
    D: Fn(&Instant<&Text>, Context<&mut State, &mut Heap>) -> bool,
  > Action<Text, State, Heap> for Prevent<T, D>
{
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    mut ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    if !(self.inner)(instant, ctx.reborrow()) {
      self.action.exec(instant, ctx)
    } else {
      None
    }
  }
}

unsafe impl<
    Text: ?Sized,
    State,
    Heap,
    T: Action<Text, State, Heap>,
    D: Fn(Accepted<&Text, &T::Value>, Context<&mut State, &mut Heap>) -> bool,
  > Action<Text, State, Heap> for Reject<T, D>
{
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    mut ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(instant, ctx.reborrow())
      .and_then(|output| {
        if (self.inner)(Accepted::new(instant, output.as_ref()), ctx) {
          None
        } else {
          output.into()
        }
      })
  }
}

unsafe impl<Text: ?Sized, State, Heap, T: Action<Text, State, Heap, Value: Default>>
  Action<Text, State, Heap> for Optional<T>
{
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    Some(self.action.exec(instant, ctx).unwrap_or_else(|| Output {
      value: Default::default(),
      digested: 0,
    }))
  }
}

unsafe impl<State, Heap, T: Action<str, State, Heap>> Action<str, State, Heap> for Boundary<T> {
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&str>,
    mut ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(instant, ctx.reborrow())
      .and_then(|output| {
        unsafe { instant.rest().get_unchecked(output.digested..) }
          .chars()
          .next()
          .map_or(true, |c| !c.is_alphanumeric() && c != '_')
          .then_some(output)
      })
  }
}

impl<T> Combinator<T> {
  /// Create a new combinator to check the [`Instant`] and [`Context`] before being executed.
  /// The combinator will be executed only if the `condition` returns `true`.
  ///
  /// This is the opposite of [`Combinator::prevent`].
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { execute: bool }
  /// # fn t(combinator: Combinator<impl Action<str, MyState>>) {
  /// combinator.when(|_, ctx| ctx.state.execute)
  /// # ;}
  /// ```
  #[inline]
  pub fn when<
    Text: ?Sized,
    State,
    Heap,
    F: Fn(&Instant<&Text>, Context<&mut State, &mut Heap>) -> bool,
  >(
    self,
    condition: F,
  ) -> Combinator<When<T, F>>
  where
    T: Action<Text, State, Heap>,
  {
    Combinator::new(When::new(self.action, condition))
  }

  /// Create a new combinator to check the [`Instant`] and [`Context`] before being executed.
  /// The combinator will reject if the `preventer` returns `true`.
  ///
  /// This is the opposite of [`Combinator::when`].
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # struct MyState { reject: bool }
  /// # fn t(combinator: Combinator<impl Action<str, MyState>>) {
  /// combinator.prevent(|_, ctx| ctx.state.reject)
  /// # ;}
  /// ```
  #[inline]
  pub fn prevent<
    Text: ?Sized,
    State,
    Heap,
    F: Fn(&Instant<&Text>, Context<&mut State, &mut Heap>) -> bool,
  >(
    self,
    preventer: F,
  ) -> Combinator<Prevent<T, F>>
  where
    T: Action<Text, State, Heap>,
  {
    Combinator::new(Prevent::new(self.action, preventer))
  }

  /// Create a new combinator to check the [`Instant`], [`Context`] and [`Output`] after being executed.
  /// The combinator will reject if the `rejecter` returns `true`.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # fn t(combinator: Combinator<impl Action>) {
  /// combinator.reject(|ctx, _| ctx.content() != "123")
  /// # ;}
  /// ```
  #[inline]
  pub fn reject<
    Text: ?Sized,
    State,
    Heap,
    F: Fn(Accepted<&Text, &T::Value>, Context<&mut State, &mut Heap>) -> bool,
  >(
    self,
    rejecter: F,
  ) -> Combinator<Reject<T, F>>
  where
    T: Action<Text, State, Heap>,
  {
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
  use crate::{
    combinator::{bytes, wrap},
    instant::Instant,
  };

  fn accepter() -> Combinator<impl Action<str, bool, Value = ()>> {
    wrap(|instant, ctx| {
      *ctx.state = true;
      instant.accept(1)
    })
  }
  fn accepter_bytes() -> Combinator<impl Action<[u8], bool, Value = ()>> {
    bytes::wrap(|instant, ctx| {
      *ctx.state = true;
      instant.accept(1)
    })
  }

  fn rejecter() -> Combinator<impl Action<str, bool, Value = ()>> {
    wrap(|_, ctx| {
      *ctx.state = true;
      None
    })
  }
  fn rejecter_bytes() -> Combinator<impl Action<[u8], bool, Value = ()>> {
    bytes::wrap(|_, ctx| {
      *ctx.state = true;
      None
    })
  }

  #[test]
  fn combinator_when() {
    let mut executed = false;
    assert!(accepter()
      .when(|_, _| false)
      .exec(
        &Instant::new("123"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      )
      .is_none());
    assert!(!executed);
    let mut executed = false;
    assert!(accepter_bytes()
      .when(|_, _| false)
      .exec(
        &Instant::new(b"123"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      )
      .is_none());
    assert!(!executed);

    let mut executed = false;
    assert!(accepter()
      .when(|_, _| true)
      .exec(
        &Instant::new("123"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      )
      .is_some());
    assert!(executed);
    let mut executed = false;
    assert!(accepter_bytes()
      .when(|_, _| true)
      .exec(
        &Instant::new(b"123"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      )
      .is_some());
    assert!(executed);
  }

  #[test]
  fn combinator_prevent() {
    let mut executed = false;
    assert!(accepter()
      .prevent(|_, _| true)
      .exec(
        &Instant::new("123"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      )
      .is_none());
    assert!(!executed);
    let mut executed = false;
    assert!(accepter_bytes()
      .prevent(|_, _| true)
      .exec(
        &Instant::new(b"123"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      )
      .is_none());
    assert!(!executed);

    let mut executed = false;
    assert!(accepter()
      .prevent(|_, _| false)
      .exec(
        &Instant::new("123"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      )
      .is_some());
    assert!(executed);
    let mut executed = false;
    assert!(accepter_bytes()
      .prevent(|_, _| false)
      .exec(
        &Instant::new(b"123"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      )
      .is_some());
    assert!(executed);
  }

  #[test]
  fn combinator_reject() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .reject(|accept, _| accept.content() != "1")
        .exec(
          &Instant::new("123"),
          Context {
            state: &mut executed,
            heap: &mut ()
          }
        )
        .unwrap()
        .digested,
      1
    );
    assert!(executed);
    let mut executed = false;
    assert_eq!(
      accepter_bytes()
        .reject(|accept, _| accept.content() != b"1")
        .exec(
          &Instant::new(b"123"),
          Context {
            state: &mut executed,
            heap: &mut ()
          }
        )
        .unwrap()
        .digested,
      1
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter().reject(|accept, _| accept.content() == "1").exec(
        &Instant::new("123"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      ),
      None
    );
    assert!(executed);
    let mut executed = false;
    assert_eq!(
      accepter_bytes()
        .reject(|accept, _| accept.content() == b"1")
        .exec(
          &Instant::new(b"123"),
          Context {
            state: &mut executed,
            heap: &mut ()
          }
        ),
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
        .exec(
          &Instant::new("123"),
          Context {
            state: &mut executed,
            heap: &mut ()
          }
        )
        .unwrap()
        .digested,
      1
    );
    assert!(executed);
    let mut executed = false;
    assert_eq!(
      accepter_bytes()
        .optional()
        .exec(
          &Instant::new(b"123"),
          Context {
            state: &mut executed,
            heap: &mut ()
          }
        )
        .unwrap()
        .digested,
      1
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      rejecter()
        .optional()
        .exec(
          &Instant::new("123"),
          Context {
            state: &mut executed,
            heap: &mut ()
          }
        )
        .unwrap()
        .digested,
      0
    );
    assert!(executed);
    let mut executed = false;
    assert_eq!(
      rejecter_bytes()
        .optional()
        .exec(
          &Instant::new(b"123"),
          Context {
            state: &mut executed,
            heap: &mut ()
          }
        )
        .unwrap()
        .digested,
      0
    );
    assert!(executed);
  }

  #[test]
  fn optional_can_be_the_last_one() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .optional()
        .exec(
          &Instant::new(""),
          Context {
            state: &mut executed,
            heap: &mut ()
          }
        )
        .unwrap()
        .digested,
      0
    );
    assert!(executed);
    let mut executed = false;
    assert_eq!(
      accepter_bytes()
        .optional()
        .exec(
          &Instant::new(b""),
          Context {
            state: &mut executed,
            heap: &mut ()
          }
        )
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
        .exec(
          &Instant::new("1"),
          Context {
            state: &mut executed,
            heap: &mut ()
          }
        )
        .unwrap()
        .digested,
      1
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter().boundary().exec(
        &Instant::new("12"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      ),
      None
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter().boundary().exec(
        &Instant::new("1a"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      ),
      None
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter().boundary().exec(
        &Instant::new("1_"),
        Context {
          state: &mut executed,
          heap: &mut ()
        }
      ),
      None
    );
    assert!(executed);
  }
}
