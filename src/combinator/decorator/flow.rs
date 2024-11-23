//! Decorators that modify the acceptance of a combinator.

use super::AcceptedContext;
use crate::{
  combinator::{wrap, Combinator, Input, Output, Parse},
  Combinator,
};

impl<T: Parse<State, Heap>, State, Heap> Combinator<T, State, Heap> {
  /// Create a new combinator to check the [`Input`] before being executed.
  /// The combinator will reject if the `preventer` returns `true`.
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
    preventer: impl Fn(&mut Input<&mut State, &mut Heap>) -> bool,
  ) -> Combinator!(T::Kind, State, Heap) {
    wrap(move |input| {
      if preventer(input) {
        None
      } else {
        self.parse(input)
      }
    })
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
      AcceptedContext<&mut Input<'text, &mut State, &mut Heap>, &Output<'text, T::Kind>>,
    ) -> bool,
  ) -> Combinator!(T::Kind, State, Heap) {
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
  /// with the default kind and zero digested if the original combinator rejects.
  /// # Caveats
  /// This requires the `Kind` to implement [`Default`],
  /// thus usually used before setting a custom kind.
  /// ```
  /// # use whitehole::Combinator;
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn t(combinator: Combinator!()) {
  /// // make the combinator optional before binding a kind
  /// combinator.optional().bind(MyKind::A)
  /// // instead of
  /// // combinator.bind(MyKind::A).optional()
  /// # ;}
  /// ```
  /// Or you can wrap `Kind` with [`Option`] to make it optional
  /// after setting a custom kind.
  /// ```
  /// # use whitehole::Combinator;
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn t(combinator: Combinator!()) {
  /// combinator.bind(Some(MyKind::A)).optional()
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
  pub fn optional(self) -> Combinator!(T::Kind, State, Heap)
  where
    T::Kind: Default,
  {
    wrap(move |input| {
      Some(self.parse(input).unwrap_or_else(|| Output {
        kind: Default::default(),
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
  pub fn boundary(self) -> Combinator!(T::Kind, State, Heap) {
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
        kind: (),
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
        kind: (),
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
        kind: (),
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
        kind: (),
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
      Some(Output { kind: (), rest: "" })
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
