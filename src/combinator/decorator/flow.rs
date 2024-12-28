//! Decorators that modify the acceptance of a combinator.

use super::AcceptedContext;
use crate::{
  combinator::{wrap_unchecked, Action, Combinator, Input, Output},
  C,
};

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
  pub fn when(self, condition: impl Fn(Input<&mut T::State, &mut T::Heap>) -> bool) -> C!(@T) {
    unsafe {
      wrap_unchecked(move |mut input| {
        if condition(input.reborrow()) {
          self.exec(input)
        } else {
          None
        }
      })
    }
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
  pub fn prevent(self, preventer: impl Fn(Input<&mut T::State, &mut T::Heap>) -> bool) -> C!(@T) {
    self.when(move |input| !preventer(input))
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
  pub fn reject(
    self,
    rejecter: impl for<'text> Fn(
      AcceptedContext<Input<'text, &mut T::State, &mut T::Heap>, &Output<T::Value>>,
    ) -> bool,
  ) -> C!(@T) {
    unsafe {
      wrap_unchecked(move |mut input| {
        self.exec(input.reborrow()).and_then(|output| {
          if rejecter(AcceptedContext {
            input,
            output: &output,
          }) {
            None
          } else {
            output.into()
          }
        })
      })
    }
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
  pub fn optional(self) -> C!(@T)
  where
    T::Value: Default,
  {
    unsafe {
      wrap_unchecked(move |input| {
        Some(self.exec(input).unwrap_or_else(|| Output {
          value: Default::default(),
          digested: 0,
        }))
      })
    }
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
  pub fn boundary(self) -> C!(@T) {
    self.reject(|ctx| {
      ctx
        .rest()
        .chars()
        .next()
        .map_or(false, |c| c.is_alphanumeric() || c == '_')
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::wrap;

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
      .exec(Input::new("123", 0, &mut executed, &mut ()).unwrap())
      .is_none());
    assert!(!executed);

    let mut executed = false;
    assert!(accepter()
      .when(|_| true)
      .exec(Input::new("123", 0, &mut executed, &mut ()).unwrap())
      .is_some());
    assert!(executed);
  }

  #[test]
  fn combinator_prevent() {
    let mut executed = false;
    assert!(accepter()
      .prevent(|_| true)
      .exec(Input::new("123", 0, &mut executed, &mut ()).unwrap())
      .is_none());
    assert!(!executed);

    let mut executed = false;
    assert!(accepter()
      .prevent(|_| false)
      .exec(Input::new("123", 0, &mut executed, &mut ()).unwrap())
      .is_some());
    assert!(executed);
  }

  #[test]
  fn combinator_reject() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .reject(|input| input.content() != "1")
        .exec(Input::new("123", 0, &mut executed, &mut ()).unwrap())
        .unwrap()
        .digested,
      1
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .reject(|input| input.content() == "1")
        .exec(Input::new("123", 0, &mut executed, &mut ()).unwrap()),
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
        .exec(Input::new("123", 0, &mut executed, &mut ()).unwrap())
        .unwrap()
        .digested,
      1
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      rejecter()
        .optional()
        .exec(Input::new("123", 0, &mut executed, &mut ()).unwrap())
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
        .exec(Input::new("1", 0, &mut executed, &mut ()).unwrap())
        .unwrap()
        .digested,
      1
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .boundary()
        .exec(Input::new("12", 0, &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .boundary()
        .exec(Input::new("1a", 0, &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .boundary()
        .exec(Input::new("1_", 0, &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);
  }
}
