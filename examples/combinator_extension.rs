use whitehole::{
  combinator::{eat, Combinator},
  parse::{Input, Parse},
  Combinator,
};

// Define a trait to extend the combinator.

pub trait CombinatorExt<T: Parse> {
  /// Create a new combinator to print the range and content if accepted.
  fn print(self) -> Combinator!(@T);
}

impl<T: Parse> CombinatorExt<T> for Combinator<T> {
  fn print(self) -> Combinator!(@T) {
    self.then(|ctx| println!("{}..{}: {:?}", ctx.input.start(), ctx.end(), ctx.content()))
  }
}

fn main() {
  eat("hello")
    .print()
    .parse(&mut Input::new("hello world", 0, &mut (), &mut ()).unwrap());
}
