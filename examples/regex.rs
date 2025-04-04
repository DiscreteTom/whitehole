use regex::Regex;
use whitehole::{
  action::Action,
  combinator::{wrap_unchecked, Combinator},
};

/// Create a combinator from a regex.
pub fn regex(s: &str) -> Combinator<impl Action<Text = str, State = (), Heap = (), Value = ()>> {
  let re = Regex::new(s).unwrap();
  unsafe {
    wrap_unchecked(move |input| {
      re.find(input.instant.rest())
        .map(|m| input.instant.accept_unchecked(m.len()))
    })
  }
}

fn main() {}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole::{action::Action, parser::Parser};

  fn helper(
    action: impl Action<Text = str, State = (), Heap = (), Value = ()>,
    input: &str,
    digested: usize,
  ) {
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
  fn test_regex() {
    let r = regex(r"\d+");
    helper(&r, "123", 3);
    helper(&r, "123abc", 3);
  }
}
