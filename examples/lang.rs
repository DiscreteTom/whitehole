//! Programming language related combinator examples.

use whitehole::{
  combinator::{eat, next, till},
  C,
};

/// Eat one or more unicode whitespaces.
/// # Caveats
/// Unicode whitespaces is a huge set so this combinator may not be efficient.
/// If you don't need to handle all unicode whitespaces, consider using a custom combinator.
/// E.g. for JSON, you can use `next(in_str!(" \t\r\n")) * (1..)` which is faster than this.
pub fn whitespaces<State, Heap>() -> C!((), State, Heap) {
  next(|c| c.is_whitespace()) * (1..)
}

pub fn singleline_comment<State, Heap>() -> C!((), State, Heap) {
  eat("//") + (till('\n') | till(()))
}

pub fn multiline_comment<State, Heap>() -> C!((), State, Heap) {
  eat("/*") + (till("*/") | till(()))
}

fn main() {}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole::action::{Action, Input};

  #[test]
  fn test_whitespaces() {
    assert_eq!(
      whitespaces()
        .parse(&mut Input::new(" \t\r\n", 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .rest,
      ""
    );
    assert_eq!(
      whitespaces()
        .parse(&mut Input::new(" \t\r\n123", 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .rest,
      "123"
    );
  }

  #[test]
  fn test_comments() {
    assert_eq!(
      singleline_comment()
        .parse(&mut Input::new("//123", 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .rest,
      ""
    );
    assert_eq!(
      singleline_comment()
        .parse(&mut Input::new("//123\n", 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .rest,
      ""
    );
    assert_eq!(
      singleline_comment()
        .parse(&mut Input::new("//123\n456", 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .rest,
      "456"
    );

    assert_eq!(
      multiline_comment()
        .parse(&mut Input::new("/*123", 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .rest,
      ""
    );
    assert_eq!(
      multiline_comment()
        .parse(&mut Input::new("/*123\n*/", 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .rest,
      ""
    );
    assert_eq!(
      multiline_comment()
        .parse(&mut Input::new("/*123\n*/456", 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .rest,
      "456"
    );
  }
}
