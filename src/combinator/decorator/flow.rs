//! Decorators that modify the acceptance of a combinator.

use super::AcceptedContext;
use crate::{
  combinator::{wrap, Combinator, Input, Output, Parse},
  Combinator,
};

impl<T: Parse> Combinator<T> {
  /// Create a new combinator to check the [`Input`] before being executed.
  /// The combinator will be executed only if the `condition` returns `true`.
  ///
  /// This is the opposite of [`Combinator::prevent`].
  /// # Examples
  /// ```
  /// # use whitehole::Combinator;
  /// # struct MyState { execute: bool }
  /// # fn t(combinator: Combinator!((), MyState)) {
  /// combinator.when(|input| input.state.execute)
  /// # ;}
  /// ```
  #[inline]
  pub fn when(
    self,
    condition: impl Fn(&mut Input<&mut T::State, &mut T::Heap>) -> bool,
  ) -> Combinator!(@T) {
    wrap(move |input| {
      if condition(input) {
        self.parse(input)
      } else {
        None
      }
    })
  }

  /// Create a new combinator to check the [`Input`] before being executed.
  /// The combinator will reject if the `preventer` returns `true`.
  ///
  /// This is the opposite of [`Combinator::when`].
  /// # Examples
  /// ```
  /// # use whitehole::Combinator;
  /// # struct MyState { reject: bool }
  /// # fn t(combinator: Combinator!((), MyState)) {
  /// combinator.prevent(|input| input.state.reject)
  /// # ;}
  /// ```
  #[inline]
  pub fn prevent(
    self,
    preventer: impl Fn(&mut Input<&mut T::State, &mut T::Heap>) -> bool,
  ) -> Combinator!(@T) {
    self.when(move |input| !preventer(input))
  }

  /// Create a new combinator to check the [`Input`] and [`Output`] after being executed.
  /// The combinator will reject if the `rejecter` returns `true`.
  /// # Examples
  /// ```
  /// # use whitehole::Combinator;
  /// # fn t(combinator: Combinator!()) {
  /// combinator.reject(|ctx| ctx.content() != "123")
  /// # ;}
  /// ```
  #[inline]
  pub fn reject(
    self,
    rejecter: impl for<'text> Fn(
      AcceptedContext<&mut Input<'text, &mut T::State, &mut T::Heap>, &Output<'text, T::Value>>,
    ) -> bool,
  ) -> Combinator!(@T) {
    wrap(move |input| {
      self.parse(input).and_then(|output| {
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

  /// Make the combinator optional.
  ///
  /// Under the hood, the combinator will be accepted
  /// with the default value and zero digested if the original combinator rejects.
  /// # Caveats
  /// This requires the `Value` to implement [`Default`],
  /// thus usually used before setting a custom value.
  /// ```
  /// # use whitehole::Combinator;
  /// # struct MyValue;
  /// # fn t(combinator: Combinator!()) {
  /// // make the combinator optional before binding a value
  /// combinator.optional().bind(MyValue)
  /// // instead of
  /// // combinator.bind(MyValue).optional()
  /// # ;}
  /// ```
  /// Or you can wrap `Value` with [`Option`] to make it optional
  /// after setting a custom value.
  /// ```
  /// # use whitehole::Combinator;
  /// # struct MyValue;
  /// # fn t(combinator: Combinator!()) {
  /// combinator.bind(Some(MyValue)).optional()
  /// # ;}
  /// ```
  /// # Examples
  /// ```
  /// # use whitehole::Combinator;
  /// # fn t(combinator: Combinator!()) {
  /// combinator.optional()
  /// # ;}
  /// ```
  #[inline]
  pub fn optional(self) -> Combinator!(@T)
  where
    T::Value: Default,
  {
    wrap(move |input| {
      Some(self.parse(input).unwrap_or_else(|| Output {
        value: Default::default(),
        rest: input.rest(),
      }))
    })
  }

  /// Create a new combinator to reject after execution
  /// if the next undigested char is alphanumeric or `_`.
  /// See [`char::is_alphanumeric`].
  /// # Examples
  /// ```
  /// # use whitehole::Combinator;
  /// # fn t(combinator: Combinator!()) {
  /// combinator.boundary()
  /// # ;}
  /// ```
  #[inline]
  pub fn boundary(self) -> Combinator!(@T) {
    self.reject(|ctx| {
      ctx
        .output
        .rest
        .chars()
        .next()
        .map_or(false, |c| c.is_alphanumeric() || c == '_')
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn accepter() -> Combinator!((), bool) {
    wrap(|input| {
      *input.state = true;
      Some(Output {
        value: (),
        rest: &input.rest()[1..],
      })
    })
  }

  fn rejecter() -> Combinator!((), bool) {
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
      .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap())
      .is_none());
    assert!(!executed);

    let mut executed = false;
    assert!(accepter()
      .when(|_| true)
      .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap())
      .is_some());
    assert!(executed);
  }

  #[test]
  fn combinator_prevent() {
    let mut executed = false;
    assert!(accepter()
      .prevent(|_| true)
      .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap())
      .is_none());
    assert!(!executed);

    let mut executed = false;
    assert!(accepter()
      .prevent(|_| false)
      .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap())
      .is_some());
    assert!(executed);
  }

  #[test]
  fn combinator_reject() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .reject(|input| input.content() != "1")
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "23"
      })
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .reject(|input| input.content() == "1")
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
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
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "23"
      })
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      rejecter()
        .optional()
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "123"
      })
    );
    assert!(executed);
  }

  #[test]
  fn combinator_boundary() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .boundary()
        .parse(&mut Input::new("1", 0, &mut executed, &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ""
      })
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .boundary()
        .parse(&mut Input::new("12", 0, &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .boundary()
        .parse(&mut Input::new("1a", 0, &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .boundary()
        .parse(&mut Input::new("1_", 0, &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);
  }
}
