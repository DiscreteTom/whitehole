use super::AcceptedContext;
use crate::{
  combinator::{wrap, Combinator, Input, Output, Parse},
  Combinator,
};

impl<State, Heap, T: Parse<State, Heap>> Combinator<State, Heap, T> {
  /// Set [`Output::kind`] to a constant kind value.
  ///
  /// If your `Kind` doesn't implement the [`Clone`] trait, consider using [`Self::select`] instead.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn t(combinator: Combinator<MyKind, (), ()>) {
  /// combinator.bind(MyKind::A)
  /// # ;}
  /// ```
  pub fn bind<NewKind>(self, kind: NewKind) -> Combinator!(NewKind, State, Heap)
  where
    NewKind: Clone,
  {
    wrap(move |input| self.parse(input).map(|output| output.map(|_| kind.clone())))
  }

  /// Set [`Output::kind`] to the default kind value.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) -> Combinator<i32, (), ()> {
  /// combinator.bind_default()
  /// # }
  /// ```
  pub fn bind_default<NewKind>(self) -> Combinator!(NewKind, State, Heap)
  where
    NewKind: Default,
  {
    wrap(move |input| {
      self
        .parse(input)
        .map(|output| output.map(|_| Default::default()))
    })
  }

  /// Set [`Output::kind`] by the `selector`.
  ///
  /// Use this if you need to calculate the kind value based on the [`Input`] and [`Output`].
  /// You can consume the original [`Output`] in the `selector`.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # enum MyKind { Num(i32) }
  /// # fn t(combinator: Combinator<MyKind, (), ()>) {
  /// combinator.select(|ctx| MyKind::Num(ctx.content().parse().unwrap()))
  /// # ;}
  /// ```
  pub fn select<NewKind>(
    self,
    selector: impl for<'text> Fn(
      AcceptedContext<&mut Input<'text, &mut State, &mut Heap>, Output<'text, T::Kind>>,
    ) -> NewKind,
  ) -> Combinator!(NewKind, State, Heap) {
    wrap(move |input| {
      self.parse(input).map(|output| Output {
        rest: output.rest,
        kind: selector(AcceptedContext { input, output }),
      })
    })
  }

  /// Convert [`Output::kind`] to a new kind value.
  ///
  /// You can consume the original [`Output`] in the `converter`.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.map(|kind| Some(kind))
  /// # ;}
  /// ```
  pub fn map<NewKind>(
    self,
    converter: impl Fn(T::Kind) -> NewKind,
  ) -> Combinator!(NewKind, State, Heap) {
    wrap(move |input| self.parse(input).map(|output| output.map(&converter)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn combinator_bind() {
    assert_eq!(
      wrap(|input| Some(Output {
        kind: (),
        rest: &input.rest()[1..]
      }))
      .bind(123)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 123,
        rest: "23"
      })
    );
  }

  #[test]
  fn combinator_bind_default() {
    assert_eq!(
      wrap(|input| Some(Output {
        kind: (),
        rest: &input.rest()[1..]
      }))
      .bind_default::<i32>()
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 0,
        rest: "23"
      })
    );
  }

  #[test]
  fn combinator_select() {
    assert_eq!(
      wrap(|input| Some(Output {
        kind: (),
        rest: &input.rest()[1..]
      }))
      .select(|ctx| if ctx.content() == "1" { 1 } else { 2 })
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 1,
        rest: "23"
      })
    );
  }

  #[test]
  fn combinator_map() {
    assert_eq!(
      wrap(|input| Some(Output {
        kind: 1,
        rest: &input.rest()[1..]
      }))
      .map(Some)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: Some(1),
        rest: "23"
      })
    );
  }
}
