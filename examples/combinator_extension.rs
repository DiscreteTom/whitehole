use std::{fmt::Debug, ops::RangeTo, slice::SliceIndex};

use whitehole::{
  action::{Action, Context, Output},
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
where
  RangeTo<usize>: SliceIndex<Text, Output = Text>,
{
  fn simple_print(self) -> Combinator<impl Action<Text, State, Heap, Value = T::Value>> {
    self.then(|accept, _| {
      println!(
        "{}..{}: {:?}",
        accept.start(),
        accept.end(),
        accept.content()
      )
    })
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
where
  RangeTo<usize>: SliceIndex<Text, Output = Text>,
{
  type Value = T::Value;

  fn exec(
    &self,
    instant: Instant<&Text>,
    mut ctx: Context<&mut State, &mut Heap>,
  ) -> Option<Output<Self::Value>> {
    self
      .action
      .exec(instant.clone(), ctx.reborrow())
      .inspect(|output| {
        let start = instant.digested();
        let end = start + output.digested;
        println!(
          "{}..{}: {:?}",
          start,
          end,
          instant.rest().get(..output.digested)
        );
      })
  }
}

fn main() {
  eat("hello")
    .simple_print()
    .exec(Instant::new("hello world"), Context::default());

  eat("hello")
    .print()
    .exec(Instant::new("hello world"), Context::default());
}
