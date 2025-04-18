use std::{fmt::Debug, ops::RangeTo, slice::SliceIndex};
use whitehole::{
  action::{Action, Input, Output},
  combinator::{eat, Combinator},
  digest::Digest,
  instant::Instant,
  parser::Parser,
};

// Define a trait to extend the combinator.

// The simpler way: return `impl`.

trait SimpleCombinatorExt<T: Action, Text: ?Sized> {
  /// Create a new combinator to print the range and content if accepted.
  fn simple_print(
    self,
  ) -> Combinator<impl Action<Text = Text, State = T::State, Heap = T::Heap, Value = T::Value>>;
}

impl<T: Action<Text = Text>, Text: ?Sized + Debug + Digest> SimpleCombinatorExt<T, Text>
  for Combinator<T>
where
  RangeTo<usize>: SliceIndex<Text, Output = Text>,
{
  fn simple_print(
    self,
  ) -> Combinator<impl Action<Text = Text, State = T::State, Heap = T::Heap, Value = T::Value>> {
    self.then(|accept| {
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

unsafe impl<T: Action<Text: Digest + Debug>> Action for Print<T>
where
  RangeTo<usize>: SliceIndex<T::Text, Output = T::Text>,
{
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  fn exec(
    &self,
    mut input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.action.exec(input.reborrow()).inspect(|output| {
      let start = input.instant.digested();
      let end = start + output.digested;
      println!(
        "{}..{}: {:?}",
        start,
        end,
        input.instant.rest().get(..output.digested)
      );
    })
  }
}

fn main() {
  Parser::builder()
    .entry(eat("hello").simple_print())
    .build("hello world")
    .next();

  Parser::builder()
    .entry(eat("hello").print())
    .build("hello world")
    .next();
}
