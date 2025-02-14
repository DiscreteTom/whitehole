use std::fmt::Debug;

use whitehole::{
  action::{Action, Input, Output},
  combinator::{eat, Combinator},
  digest::Digest,
  instant::Instant,
};

// Define a trait to extend the combinator.

// The simpler way: return `impl`.

trait SimpleCombinatorExt<T: Action<Text, State, Heap>, Text: ?Sized, State, Heap> {
  /// Create a new combinator to print the range and content if accepted.
  fn simple_print(self) -> Combinator<impl Action<Text, State, Heap, Value = T::Value>>;
}

impl<T: Action<Text, State, Heap>, Text: ?Sized + Debug + Digest, State, Heap>
  SimpleCombinatorExt<T, Text, State, Heap> for Combinator<T>
{
  fn simple_print(self) -> Combinator<impl Action<Text, State, Heap, Value = T::Value>> {
    self.then(|ctx| println!("{}..{}: {:?}", ctx.start(), ctx.end(), ctx.content()))
  }
}

// The more complex way: use a custom struct.

struct Print<T> {
  action: T,
}

trait CombinatorExt<T> {
  /// Create a new combinator to print the range and content if accepted.
  fn print(self) -> Combinator<Print<T>>;
}

impl<T> CombinatorExt<T> for Combinator<T> {
  fn print(self) -> Combinator<Print<T>> {
    Combinator::new(Print {
      action: self.action,
    })
  }
}

unsafe impl<Text: ?Sized + Debug + Digest, State, Heap, T: Action<Text, State, Heap>>
  Action<Text, State, Heap> for Print<T>
{
  type Value = T::Value;

  fn exec(&self, mut input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.action.exec(input.reborrow()).inspect(|output| {
      let start = input.instant().digested();
      let end = start + output.digested;
      println!("{}..{}: {:?}", start, end, unsafe {
        input.instant().rest().span_unchecked(output.digested)
      });
    })
  }
}

fn main() {
  eat("hello")
    .simple_print()
    .exec(Input::new(Instant::new("hello world"), &mut (), &mut ()));

  eat("hello")
    .print()
    .exec(Input::new(Instant::new("hello world"), &mut (), &mut ()));
}
