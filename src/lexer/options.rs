use super::{
  expectation::Expectation,
  fork::{ForkDisabled, ForkEnabled},
  re_lex::ReLexContext,
};

pub struct LexOptions<'expect_literal, Kind: 'static, Fork> {
  pub expectation: Expectation<'expect_literal, Kind>,
  /// See [`LexOptions::fork()`].
  pub fork: Fork,
  /// See [`LexOptions::re_lex()`].
  pub re_lex: ReLexContext,
}

impl<'expect_literal, Kind: 'static> Default for LexOptions<'expect_literal, Kind, ForkDisabled> {
  fn default() -> Self {
    Self {
      expectation: Expectation::default(),
      fork: ForkDisabled,
      re_lex: ReLexContext::default(),
    }
  }
}

impl<'expect_literal, Kind: 'static> From<Expectation<'expect_literal, Kind>>
  for LexOptions<'expect_literal, Kind, ForkDisabled>
{
  fn from(expectation: Expectation<'expect_literal, Kind>) -> Self {
    Self::default().expect(expectation)
  }
}
impl<'expect_literal, Kind: 'static> From<ReLexContext>
  for LexOptions<'expect_literal, Kind, ForkDisabled>
{
  fn from(re_lex: ReLexContext) -> Self {
    Self::default().re_lex(re_lex)
  }
}

impl<'expect_literal, Kind: 'static, Fork> LexOptions<'expect_literal, Kind, Fork> {
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_literal, Kind>>) -> Self {
    self.expectation = expectation.into();
    self
  }

  pub fn expect_with(
    mut self,
    f: impl FnOnce(Expectation<'expect_literal, Kind>) -> Expectation<'expect_literal, Kind>,
  ) -> Self {
    self.expectation = f(Expectation::default());
    self
  }

  /// If set, and the lexing is re-lexable (the accepted action is not the last candidate action),
  /// the [`LexOutput::re_lex`](crate::lexer::output::LexOutput::re_lexable) will be `Some`.
  // TODO: example
  pub fn fork(self) -> LexOptions<'expect_literal, Kind, ForkEnabled> {
    LexOptions {
      expectation: self.expectation,
      fork: ForkEnabled,
      re_lex: self.re_lex,
    }
  }

  // TODO: comments
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = re_lex;
    self
  }
}
