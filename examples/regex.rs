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
    action::{Action, Context, Input},
    instant::Instant,
  };

  #[test]
  fn test_regex() {
    assert_eq!(
      regex(r"\d+")
        .exec(Instant::new("123"), Context::default())
        .unwrap()
        .digested,
      3
    );
    assert_eq!(
      regex(r"\d+")
        .exec(Instant::new("123abc"), Context::default())
        .unwrap()
        .digested,
      3
    );
  }
}
