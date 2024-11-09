use super::AcceptedOutputContext;
use crate::combinator::{Combinator, Input, Output};

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Combinator<'a, Kind, State, Heap> {
  /// Set [`Output::kind`] to a constant kind value.
  ///
  /// If your `Kind` doesn't implement the [`Clone`] trait, consider using [`Self::select`] instead.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # enum MyKind { A }
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.bind(MyKind::A)
  /// # ;}
  /// ```
  pub fn bind<NewKind>(self, kind: NewKind) -> Combinator<'a, NewKind, State, Heap>
  where
    NewKind: Clone + 'a,
  {
    Combinator::boxed(move |input| self.parse(input).map(|output| output.map(|_| kind.clone())))
  }

  /// Set [`Output::kind`] to the default kind value.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.bind_default()
  /// # ;}
  /// ```
  pub fn bind_default<NewKind>(self) -> Combinator<'a, NewKind, State, Heap>
  where
    NewKind: Default,
  {
    Combinator::boxed(move |input| {
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
  /// # enum MyKind { A, B }
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.select(|ctx| if ctx.content() == "A" { MyKind::A } else { MyKind::B })
  /// # ;}
  /// ```
  pub fn select<NewKind>(
    self,
    selector: impl Fn(AcceptedOutputContext<&mut Input<&mut State, &mut Heap>, Output<Kind>>) -> NewKind
      + 'a,
  ) -> Combinator<'a, NewKind, State, Heap>
  where
    NewKind: Default,
  {
    Combinator::boxed(move |input| {
      self.parse(input).map(|output| Output {
        digested: output.digested,
        kind: selector(AcceptedOutputContext { input, output }),
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
    converter: impl Fn(Kind) -> NewKind + 'a,
  ) -> Combinator<'a, NewKind, State, Heap> {
    Combinator::boxed(move |input| self.parse(input).map(|output| output.map(&converter)))
  }
}
