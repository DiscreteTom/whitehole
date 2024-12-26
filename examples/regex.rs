use regex::Regex;
use whitehole::{combinator::eater_unchecked, C};

/// Create a combinator from a regex.
pub fn regex<State, Heap>(s: &str) -> C!((), State, Heap) {
  let re = Regex::new(s).unwrap();
  unsafe { eater_unchecked(move |input| re.find(input.rest()).map(|m| m.len()).unwrap_or(0)) }
}

fn main() {}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole::action::{Input, Action};

  #[test]
  fn test_regex() {
    assert_eq!(
      regex(r"\d+")
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .rest,
      ""
    );
    assert_eq!(
      regex(r"\d+")
        .parse(&mut Input::new("123abc", 0, &mut (), &mut ()).unwrap())
        .unwrap()
        .rest,
      "abc"
    );
  }
}
