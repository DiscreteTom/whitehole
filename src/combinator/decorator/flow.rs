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
  /// # Caveats
  /// This requires the `Kind` to implement [`Default`],
  /// thus usually used before setting a custom kind.
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// // bind a kind after calling `optional`
  /// combinator.optional().bind(MyKind::A)
  /// // instead of
  /// // combinator.bind(MyKind::A).optional()
  /// # ;}
  /// ```
  /// Or you can wrap `Kind` with [`Option`]:
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.bind(Some(MyKind::A)).optional()
  /// # ;}
  /// ```
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.optional()
  /// # ;}
  /// ```
  pub fn optional(self) -> Self
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

  /// Reject the combinator if the `condition` returns `true`.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.reject(|ctx| ctx.content() != "123")
  /// # ;}
  /// ```
  pub fn reject(
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
  fn combinator_optional() {
    let mut executed = false;
    assert_eq!(
      accepter()
        .optional()
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
        .optional()
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
      Some(Output {
        kind: (),
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
        .reject(|_| false)
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
        .reject(|_| true)
        .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
      None
    );
    assert!(executed);
  }
}
