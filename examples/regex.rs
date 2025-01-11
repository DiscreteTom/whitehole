use regex::Regex;
use whitehole::{
  action::Action,
  combinator::{wrap_unchecked, Combinator},
};

/// Create a combinator from a regex.
pub fn regex<State, Heap>(s: &str) -> Combinator<impl Action<State, Heap, Value = ()>> {
  let re = Regex::new(s).unwrap();
  unsafe {
    wrap_unchecked(move |input| {
      re.find(input.instant().rest())
        .map(|m| input.digest_unchecked(m.len()))
    })
  }
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
  fn test_regex() {
    assert_eq!(
      regex(r"\d+")
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
        .unwrap()
        .digested,
      3
    );
    assert_eq!(
      regex(r"\d+")
        .exec(Input::new(Instant::new("123abc"), &mut (), &mut ()))
        .unwrap()
        .digested,
      3
    );
  }
}
