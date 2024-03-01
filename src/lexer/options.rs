use super::{expectation::Expectation, output::ReLexContext};

pub struct LexOptions<'expect_text, Kind: 'static> {
  pub expectation: Expectation<'expect_text, Kind>,
  /// If `true`, the [`LexOutput::re_lex`](crate::lexer::output::LexOutput::re_lex) might be `Some`.
  pub fork: bool,
  /// Provide this if the lex is a re-lex.
  pub re_lex: Option<ReLexContext>,
}

impl<'expect_text, Kind: 'static> Default for LexOptions<'expect_text, Kind> {
  fn default() -> Self {
    Self {
      expectation: Expectation::default(),
      fork: false,
      re_lex: None,
    }
  }
}

impl<'expect_text, Kind: 'static> From<Expectation<'expect_text, Kind>>
  for LexOptions<'expect_text, Kind>
{
  fn from(expectation: Expectation<'expect_text, Kind>) -> Self {
    Self::default().expect(expectation)
  }
}

impl<'expect_text, Kind: 'static> From<ReLexContext> for LexOptions<'expect_text, Kind> {
  fn from(re_lex: ReLexContext) -> Self {
    Self::default().re_lex(re_lex)
  }
}

impl<'expect_text, Kind: 'static> LexOptions<'expect_text, Kind> {
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_text, Kind>>) -> Self {
    self.expectation = expectation.into();
    self
  }
  pub fn fork(mut self) -> Self {
    self.fork = true;
    self
  }
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = Some(re_lex);
    self
  }
}
