//! Building block of a parser.

mod decorator;
mod input;
mod operator;
mod output;

pub use decorator::*;
pub use input::*;
pub use output::*;

/// A boxed function. Return [`None`] if the combinator is rejected.
pub type CombinatorExec<'a, Kind, State = (), Heap = ()> =
  Box<dyn Fn(&mut Input<&mut State, &mut Heap>) -> Option<Output<Kind>> + 'a>;

/// Building block of a parser.
pub struct Combinator<'a, Kind, State = (), Heap = ()> {
  exec: CombinatorExec<'a, Kind, State, Heap>,
}

impl<'a, Kind, State, Heap> Combinator<'a, Kind, State, Heap> {
  /// Create a new instance.
  pub fn new(exec: CombinatorExec<'a, Kind, State, Heap>) -> Self {
    Self { exec }
  }

  /// Create a new instance by boxing the `exec` function.
  pub fn boxed(
    exec: impl Fn(&mut Input<&mut State, &mut Heap>) -> Option<Output<Kind>> + 'a,
  ) -> Self {
    Self::new(Box::new(exec))
  }

  /// Execute the combinator.
  pub fn parse(&self, input: &mut Input<&mut State, &mut Heap>) -> Option<Output<Kind>> {
    (self.exec)(input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn combinator_parse() {
    assert_eq!(
      Combinator::boxed(|_| Some(Output {
        kind: (),
        digested: 1
      }))
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 1
      })
    );
  }
}
