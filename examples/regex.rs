use regex::Regex;
use whitehole::{
  combinator::eater_unchecked,
  parse::{Input, Parse},
  Combinator,
};

/// Create a combinator from a regex.
fn regex<State, Heap>(s: &str) -> Combinator!((), State, Heap) {
  let re = Regex::new(s).unwrap();
  unsafe { eater_unchecked(move |input| re.find(input.rest()).map(|m| m.len()).unwrap_or(0)) }
}

fn main() {
  assert_eq!(
    regex(r"\d+")
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .unwrap()
      .rest,
    ""
  );
}
