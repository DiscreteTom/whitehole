use super::expectation::Expectation;

// this should never be constructed by user
// and the fields should never be accessed by user
// because the `action_index` is an internal index.
// so we make fields only public for crate.
#[derive(Clone, Debug)]
pub struct ReLexContext {
  /// Re-lex is effective only if the
  /// [`ActionInput::start`](crate::lexer::action::ActionInput::start)
  /// equals to this.
  pub(crate) start: usize,
  /// How many actions are skipped.
  /// This is effective only if
  /// the [`ActionInput::start`](crate::lexer::action::ActionInput::start)
  /// equals to [`Self::start`].
  pub(crate) skip: usize,
}

impl Default for ReLexContext {
  fn default() -> Self {
    // set skip to 0 means this is not a re-lex
    Self { start: 0, skip: 0 }
  }
}

pub struct LexOptions<'expect_text, Kind> {
  pub expectation: Expectation<'expect_text, Kind>,
  /// If `true`, the [`LexOutput::re_lex`](crate::lexer::output::LexOutput::re_lex) might be `Some`.
  /// Provide this if the lex is a re-lex.
  pub re_lex: Option<ReLexContext>,
}

impl<'expect_text, Kind> Default for LexOptions<'expect_text, Kind> {
  fn default() -> Self {
    Self {
      expectation: Expectation::default(),
      re_lex: None,
    }
  }
}

impl<'expect_text, Kind> From<Expectation<'expect_text, Kind>> for LexOptions<'expect_text, Kind> {
  fn from(expectation: Expectation<'expect_text, Kind>) -> Self {
    Self::default().expect(expectation)
  }
}

impl<'expect_text, Kind> From<ReLexContext> for LexOptions<'expect_text, Kind> {
  fn from(re_lex: ReLexContext) -> Self {
    Self::default().re_lex(re_lex)
  }
}

impl<'expect_text, Kind> LexOptions<'expect_text, Kind> {
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_text, Kind>>) -> Self {
    self.expectation = expectation.into();
    self
  }
  /// Provide this if the lex is a re-lex.
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = Some(re_lex);
    self
  }
}
