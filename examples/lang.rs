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
    action::{Action, Input},
    instant::Instant,
  };

  #[test]
  fn test_whitespaces() {
    assert_eq!(
      whitespaces()
        .exec(Input::new(Instant::new(" \t\r\n"), &mut (), &mut ()))
        .unwrap()
        .digested,
      4
    );
    assert_eq!(
      whitespaces()
        .exec(Input::new(Instant::new(" \t\r\n123"), &mut (), &mut ()))
        .unwrap()
        .digested,
      4
    );
  }

  #[test]
  fn test_comments() {
    assert_eq!(
      singleline_comment()
        .exec(Input::new(Instant::new("//123"), &mut (), &mut ()))
        .unwrap()
        .digested,
      5
    );
    assert_eq!(
      singleline_comment()
        .exec(Input::new(Instant::new("//123\n"), &mut (), &mut ()))
        .unwrap()
        .digested,
      6
    );
    assert_eq!(
      singleline_comment()
        .exec(Input::new(Instant::new("//123\n456"), &mut (), &mut ()))
        .unwrap()
        .digested,
      6
    );

    assert_eq!(
      multiline_comment()
        .exec(Input::new(Instant::new("/*123"), &mut (), &mut ()))
        .unwrap()
        .digested,
      5
    );
    assert_eq!(
      multiline_comment()
        .exec(Input::new(Instant::new("/*123\n*/"), &mut (), &mut ()))
        .unwrap()
        .digested,
      8
    );
    assert_eq!(
      multiline_comment()
        .exec(Input::new(Instant::new("/*123\n*/456"), &mut (), &mut ()))
        .unwrap()
        .digested,
      8
    );
  }
}
