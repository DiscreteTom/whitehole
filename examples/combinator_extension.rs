use whitehole::{
  combinator::{eat, Combinator},
  parse::{Input, Parse},
  Combinator,
};

// Define a trait to extend the combinator.

pub trait CombinatorExt<T: Parse<State, Heap>, State, Heap> {
  /// Create a new combinator to print the range and content if accepted.
  fn print(self) -> Combinator!(T::Kind, State, Heap);
}

impl<T: Parse<State, Heap>, State, Heap> CombinatorExt<T, State, Heap>
  for Combinator<T, State, Heap>
{
  fn print(self) -> Combinator!(T::Kind, State, Heap) {
    self.then(|ctx| println!("{}..{}: {:?}", ctx.input.start(), ctx.end(), ctx.content()))
  }
}

fn main() {
  eat("hello")
    .print()
    .parse(&mut Input::new("hello world", 0, &mut (), &mut ()).unwrap());
}
