use regex::Regex;
use whitehole::{
  action::Action,
  combinator::{wrap_unchecked, Combinator},
};

/// Create a combinator from a regex.
pub fn regex<State, Heap>(s: &str) -> Combinator<impl Action<str, State, Heap, Value = ()>> {
  let re = Regex::new(s).unwrap();
  unsafe {
    wrap_unchecked(move |instant, _| {
      re.find(instant.rest())
        .map(|m| instant.accept_unchecked(m.len()))
    })
  }
}

fn main() {}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole::{
    action::{Action, Context},
    instant::Instant,
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
  fn test_regex() {
    let r = regex(r"\d+");
    helper(&r, "123", 3);
    helper(&r, "123abc", 3);
  }
}
