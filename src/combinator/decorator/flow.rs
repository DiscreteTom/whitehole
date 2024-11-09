use super::AcceptedOutputContext;
use crate::combinator::{Combinator, Input, Output};

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Combinator<'a, Kind, State, Heap> {
  /// Check the [`Input`] before the combinator is executed.
  /// Reject if the `condition` returns `true`.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.prevent(|input| input.state.reject)
  /// # ;}
  /// ```
  pub fn prevent(self, condition: impl Fn(&mut Input<&mut State, &mut Heap>) -> bool + 'a) -> Self {
    Combinator::boxed(move |input| {
      if condition(input) {
        None
      } else {
        self.parse(input)
      }
    })
  }

  /// If the combinator is rejected, accept it with the default kind and zero digested.
  ///
  /// If the `Kind` doesn't implement [`Default`], consider using [`Self::optional`] instead.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.accept()
  /// # ;}
  /// ```
  pub fn accept(self) -> Self
  where
    Kind: Default,
  {
    Combinator::boxed(move |input| {
      Some(self.parse(input).unwrap_or_else(|| Output {
        kind: Default::default(),
        digested: 0,
      }))
    })
  }

  /// If the combinator is rejected, accept it with [`None`] as the kind and zero digested.
  ///
  /// If you want to use the default kind instead of [`None`], consider using [`Self::accept`] instead.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.optional()
  /// # ;}
  /// ```
  pub fn optional(self) -> Combinator<'a, Option<Kind>, State, Heap> {
    Combinator::boxed(move |input| {
      Some(
        self
          .parse(input)
          .map(|output| output.map(Some))
          .unwrap_or_else(|| Output {
            kind: None,
            digested: 0,
          }),
      )
    })
  }

  /// Reject the combinator after execution.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.reject()
  /// # ;}
  /// ```
  pub fn reject(self) -> Self {
    Combinator::boxed(move |input| {
      self.parse(input);
      None
    })
  }

  /// Reject the combinator if the `condition` returns `true`.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.reject_if(|ctx| ctx.content() != "123")
  /// # ;}
  /// ```
  pub fn reject_if(
    self,
    condition: impl Fn(AcceptedOutputContext<&mut Input<&mut State, &mut Heap>, &Output<Kind>>) -> bool
      + 'a,
  ) -> Self {
    Combinator::boxed(move |input| {
      self.parse(input).and_then(|output| {
        if condition(AcceptedOutputContext {
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

#[cfg(test)]
mod tests {
  use super::*;

  fn accepter() -> Combinator<'static, (), bool, ()> {
    Combinator::boxed(|input| {
      *input.state = true;
      Some(Output {
        kind: (),
        digested: 1,
      })
    })
  }

  fn rejecter() -> Combinator<'static, (), bool, ()> {
    Combinator::boxed(|input| {
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
  fn combinator_accept() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .accept()
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 1
      })
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      rejecter()
        .accept()
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 0
      })
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
        kind: Some(()),
        digested: 1
      })
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      rejecter()
        .optional()
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
      Some(Output {
        kind: Option::<()>::None,
        digested: 0
      })
    );
    assert!(executed);
  }

  #[test]
  fn combinator_reject() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .reject()
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);
  }

  #[test]
  fn combinator_reject_if() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .reject_if(|_| false)
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 1
      })
    );
    assert!(executed);

    let mut executed = false;
    assert_eq!(
      accepter()
        .reject_if(|_| true)
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);
  }
}
