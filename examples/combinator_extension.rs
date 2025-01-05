use whitehole::{
  action::{Action, Input, Output},
  combinator::{eat, Combinator},
  instant::Instant,
};

// Define a trait to extend the combinator.

// The simpler way: return `impl`.

trait SimpleCombinatorExt<T: Action<State, Heap>, State, Heap> {
  /// Create a new combinator to print the range and content if accepted.
  fn simple_print(self) -> Combinator<impl Action<State, Heap, Value = T::Value>>;
}

impl<T: Action<State, Heap>, State, Heap> SimpleCombinatorExt<T, State, Heap> for Combinator<T> {
  fn simple_print(self) -> Combinator<impl Action<State, Heap, Value = T::Value>> {
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

unsafe impl<State, Heap, T: Action<State, Heap>> Action<State, Heap> for Print<T> {
  type Value = T::Value;

  fn exec(&self, mut input: Input<&mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.action.exec(input.reborrow()).inspect(|output| {
      let start = input.instant().digested();
      let end = start + output.digested;
      println!(
        "{}..{}: {:?}",
        start,
        end,
        &input.instant().text()[start..end]
      );
    })
  }
}

fn main() {
  eat("hello")
    .simple_print()
    .exec(Input::new(Instant::new("hello world"), &mut (), &mut ()).unwrap());

  eat("hello")
    .print()
    .exec(Input::new(Instant::new("hello world"), &mut (), &mut ()).unwrap());
}
