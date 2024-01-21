use super::expectation::Expectation;

#[derive(Default)]
pub struct LexerCoreLexOptions<'expect, Kind> {
  pub start: usize,
  pub expectation: Expectation<'expect, Kind>,
  pub peek: bool,
}
