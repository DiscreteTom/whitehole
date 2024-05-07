use super::{
  expectation::Expectation,
  fork::{ForkDisabled, ForkEnabled},
};

pub struct LexOptions<'expect_text, Kind: 'static, Fork> {
  pub expectation: Expectation<'expect_text, Kind>,
  /// See [`LexOptions::fork()`].
  pub fork: Fork,
}

impl<'expect_text, Kind: 'static> Default for LexOptions<'expect_text, Kind, ForkDisabled> {
  fn default() -> Self {
    Self {
      expectation: Expectation::default(),
      fork: ForkDisabled,
    }
  }
}

impl<'expect_text, Kind: 'static> From<Expectation<'expect_text, Kind>>
  for LexOptions<'expect_text, Kind, ForkDisabled>
{
  fn from(expectation: Expectation<'expect_text, Kind>) -> Self {
    Self::default().expect(expectation)
  }
}

impl<'expect_text, Kind: 'static, Fork> LexOptions<'expect_text, Kind, Fork> {
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_text, Kind>>) -> Self {
    self.expectation = expectation.into();
    self
  }

  /// If set, the [`LexOutput::re_lex`](crate::lexer::output::LexOutput::re_lex) *might* be `Some`.
  // TODO: example
  pub fn fork<ActionState>(self) -> LexOptions<'expect_text, Kind, ForkEnabled<ActionState>>
  where
    ActionState: Clone,
  {
    LexOptions {
      expectation: self.expectation,
      fork: ForkEnabled::default(),
    }
  }
}
