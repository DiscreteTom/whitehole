use super::AcceptedOutputContext;
use crate::combinator::{Combinator, Input, Output};

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Combinator<'a, Kind, State, Heap> {
  /// Modify `State` and `Heap` before the combinator is executed.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.prepare(|input| input.state.value += 1)
  /// # ;}
  /// ```
  pub fn prepare(self, modifier: impl Fn(&mut Input<&mut State, &mut Heap>) + 'a) -> Self {
    Combinator::boxed(move |input| {
      modifier(input);
      self.parse(input)
    })
  }

  /// Modify `State` and `Heap` if the combinator is accepted.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.then(|ctx| ctx.input.state.value += 1)
  /// # ;}
  /// ```
  pub fn then(
    self,
    modifier: impl Fn(AcceptedOutputContext<&mut Input<&mut State, &mut Heap>, &Output<Kind>>) + 'a,
  ) -> Self {
    Combinator::boxed(move |input| {
      self.parse(input).inspect(|output| {
        modifier(AcceptedOutputContext { input, output });
      })
    })
  }

  /// Modify `State` and `Heap` if the combinator is rejected.
  /// # Examples
  /// ```
  /// # use whitehole::combinator::Combinator;
  /// # fn t(combinator: Combinator<(), (), ()>) {
  /// combinator.rollback(|input| input.state.value += 1)
  /// # ;}
  /// ```
  pub fn rollback(self, modifier: impl Fn(&mut Input<&mut State, &mut Heap>) + 'a) -> Self {
    Combinator::boxed(move |input| {
      let output = self.parse(input);
      if output.is_none() {
        modifier(input);
      }
      output
    })
  }
}
