//! Programming language related combinator examples.

use whitehole::{
  combinator::{eat, till},
  Combinator,
};

pub fn singleline_comment<State, Heap>() -> Combinator!((), State, Heap) {
  eat("//") + (till('\n') | till(()))
}

pub fn multiline_comment<State, Heap>() -> Combinator!((), State, Heap) {
  eat("/*") + (till("*/") | till(()))
}

fn main() {}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole::parse::{Input, Parse};

  #[test]
  fn comments() {
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
