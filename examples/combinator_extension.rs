use whitehole::{
  action::{Action, Input},
  combinator::{eat, Combinator},
  C,
};

// Define a trait to extend the combinator.

pub trait CombinatorExt<T: Action> {
  /// Create a new combinator to print the range and content if accepted.
  fn print(self) -> C!(@T);
}

impl<T: Action> CombinatorExt<T> for Combinator<T> {
  fn print(self) -> C!(@T) {
    self.then(|ctx| println!("{}..{}: {:?}", ctx.input.start(), ctx.end(), ctx.content()))
  }
}

fn main() {
  eat("hello")
    .print()
    .exec(Input::new("hello world", 0, &mut (), &mut ()).unwrap());
}
