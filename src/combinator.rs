mod input;

pub use input::*;

pub trait Combinator<Kind, State, Heap> {
  fn parse(&self, input: Input<&mut State, &mut Heap>) -> Option<(usize, Kind)>;
}

impl<F, Kind, State, Heap> Combinator<Kind, State, Heap> for F
where
  F: Fn(Input<&mut State, &mut Heap>) -> Option<(usize, Kind)>,
{
  fn parse(&self, input: Input<&mut State, &mut Heap>) -> Option<(usize, Kind)> {
    self(input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn _fn_as_combinator() -> impl Combinator<(), (), ()> {
    fn parse(_: Input<&mut (), &mut ()>) -> Option<(usize, ())> {
      None
    }
    parse
  }

  fn _closure_as_combinator() -> impl Combinator<(), (), ()> {
    |_: Input<&mut (), &mut ()>| None
  }
}
