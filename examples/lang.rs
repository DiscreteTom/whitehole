//! Programming language related combinator examples.

use whitehole::{
  action::Action,
  combinator::{eat, next, till, Combinator},
};

/// Eat one or more unicode whitespaces.
/// # Caveats
/// Unicode whitespaces is a huge set so this combinator may not be efficient.
/// If you don't need to handle all unicode whitespaces, consider using a custom combinator.
/// E.g. for JSON, you can use `next(in_str!(" \t\r\n")) * (1..)` which is faster than this.
pub fn whitespaces<State, Heap>() -> Combinator<impl Action<str, State, Heap, Value = ()>> {
  next(|c| c.is_whitespace()) * (1..)
}

pub fn singleline_comment<State, Heap>() -> Combinator<impl Action<str, State, Heap, Value = ()>> {
  eat("//") + (till('\n') | till(()))
}

pub fn multiline_comment<State, Heap>() -> Combinator<impl Action<str, State, Heap, Value = ()>> {
  eat("/*") + (till("*/") | till(()))
}

fn main() {}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole::{
    action::{Action, Context},
    instant::Instant,
    parser::Parser,
  };

  fn helper(action: impl Action<Value = ()>, input: &str, digested: usize) {
    assert_eq!(
      Parser::builder()
        .entry(action)
        .build(input)
        .next()
        .unwrap()
        .digested,
      digested
    )
  }

  #[test]
  fn test_whitespaces() {
    let ws = whitespaces();
    helper(&ws, " \t\r\n", 4);
    helper(&ws, " \t\r\n123", 4);
  }

  #[test]
  fn test_comments() {
    let single = singleline_comment();
    helper(&single, "//123", 5);
    helper(&single, "//123\n", 6);
    helper(&single, "//123\n456", 6);

    let multi = multiline_comment();
    helper(&multi, "/*123", 5);
    helper(&multi, "/*123\n*/", 8);
    helper(&multi, "/*123\n*/456", 8);
  }
}
