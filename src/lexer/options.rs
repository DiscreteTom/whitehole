use super::{
  expectation::Expectation,
  fork::{ForkDisabled, ForkEnabled},
  re_lex::ReLexContext,
};

pub struct LexOptions<'expect_literal, Kind: 'static, ErrAcc, Fork> {
  pub expectation: Expectation<'expect_literal, Kind>,
  /// See [`Self::err_acc`]
  pub err_acc: ErrAcc,
  /// See [`LexOptions::fork()`].
  pub fork: Fork,
  /// See [`LexOptions::re_lex()`].
  pub re_lex: ReLexContext,
}

impl<'expect_literal, Kind: 'static> LexOptions<'expect_literal, Kind, (), ForkDisabled> {
  pub fn new() -> Self {
    Self {
      expectation: Expectation::default(),
      err_acc: (),
      fork: ForkDisabled,
      re_lex: ReLexContext::default(),
    }
  }
}

impl<'expect_literal, Kind: 'static> From<Expectation<'expect_literal, Kind>>
  for LexOptions<'expect_literal, Kind, (), ForkDisabled>
{
  fn from(expectation: Expectation<'expect_literal, Kind>) -> Self {
    Self::new().expect(expectation)
  }
}
impl<'expect_literal, Kind: 'static> From<ReLexContext>
  for LexOptions<'expect_literal, Kind, (), ForkDisabled>
{
  fn from(re_lex: ReLexContext) -> Self {
    Self::new().re_lex(re_lex)
  }
}

impl<'expect_literal, Kind: 'static, ErrAcc, Fork> LexOptions<'expect_literal, Kind, ErrAcc, Fork> {
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

  /// Set the error accumulator.
  pub fn err_acc<NewErrAcc>(
    self,
    err_acc: NewErrAcc,
  ) -> LexOptions<'expect_literal, Kind, NewErrAcc, Fork> {
    LexOptions {
      expectation: self.expectation,
      err_acc,
      fork: self.fork,
      re_lex: self.re_lex,
    }
  }

  /// Collect the errors into a vector.
  pub fn errs_to_vec<E>(self) -> LexOptions<'expect_literal, Kind, Vec<E>, Fork> {
    self.err_acc(Vec::new())
  }

  /// If set, and the lexing is re-lexable (the accepted action is not the last candidate action),
  /// the [`LexOutput::re_lex`](crate::lexer::output::LexOutput::re_lexable) will be `Some`.
  // TODO: example
  pub fn fork(self) -> LexOptions<'expect_literal, Kind, ErrAcc, ForkEnabled> {
    LexOptions {
      expectation: self.expectation,
      err_acc: self.err_acc,
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
