use super::expectation::Expectation;

/// With this struct you can continue a finished lex.
/// For most cases this will be constructed by [`ForkEnabled`]
/// (when lexing with [`LexOptions::fork`] enabled).
/// You can also construct this if you implement [`LexOptionsFork`],
/// but make sure you know what you are doing.
#[derive(PartialEq, Clone, Debug)]
pub struct ReLexContext {
  /// See [`Self::skip`].
  pub start: usize,
  /// How many actions are skipped.
  /// This is effective only if
  /// the [`ActionInput::start`](crate::lexer::action::ActionInput::start)
  /// equals to [`Self::start`].
  pub skip: usize,
}

impl Default for ReLexContext {
  fn default() -> Self {
    // set skip to 0 means this is not a re-lex
    Self { start: 0, skip: 0 }
  }
}

/// See [`LexOptions::fork`].
pub trait LexOptionsFork: Default {
  type ReLexType: Default;

  fn create_re_lex_context(
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::ReLexType;
}
#[derive(Default)]
pub struct ForkEnabled;
impl LexOptionsFork for ForkEnabled {
  type ReLexType = Option<ReLexContext>;

  fn create_re_lex_context(
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::ReLexType {
    if action_index < actions_len - 1 {
      // current action is not the last one
      // so the lex is re-lex-able
      Some(ReLexContext {
        skip: action_index + 1, // index + 1 is the count of actions to skip
        start,
      })
    } else {
      // current action is the last one
      // no next action to re-lex
      None
    }
  }
}
#[derive(Default)]
pub struct ForkDisabled;
impl LexOptionsFork for ForkDisabled {
  type ReLexType = ();

  fn create_re_lex_context(
    _start: usize,
    _actions_len: usize,
    _action_index: usize,
  ) -> Self::ReLexType {
    ()
  }
}

pub struct LexOptions<'expect_text, Kind: 'static, Fork: LexOptionsFork> {
  pub expectation: Expectation<'expect_text, Kind>,
  /// See [`LexOptions::fork()`].
  pub fork: Fork,
  /// See [`LexOptions::re_lex()`].
  pub re_lex: Option<ReLexContext>,
}

impl<'expect_text, Kind, Fork: LexOptionsFork> Default for LexOptions<'expect_text, Kind, Fork> {
  fn default() -> Self {
    Self {
      expectation: Expectation::default(),
      fork: Fork::default(),
      re_lex: None,
    }
  }
}

impl<'expect_text, Kind, Fork: LexOptionsFork> From<Expectation<'expect_text, Kind>>
  for LexOptions<'expect_text, Kind, Fork>
{
  fn from(expectation: Expectation<'expect_text, Kind>) -> Self {
    Self::default().expect(expectation)
  }
}

impl<'expect_text, Kind, Fork: LexOptionsFork> From<ReLexContext>
  for LexOptions<'expect_text, Kind, Fork>
{
  fn from(re_lex: ReLexContext) -> Self {
    Self::default().re_lex(re_lex)
  }
}

impl<'expect_text, Kind, Fork: LexOptionsFork> LexOptions<'expect_text, Kind, Fork> {
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_text, Kind>>) -> Self {
    self.expectation = expectation.into();
    self
  }

  /// If set, the [`LexOutput::re_lex`](crate::lexer::output::LexOutput::re_lex) *might* be `Some`.
  // TODO: example
  pub fn fork(self) -> LexOptions<'expect_text, Kind, ForkEnabled> {
    LexOptions {
      expectation: self.expectation,
      fork: ForkEnabled,
      re_lex: self.re_lex,
    }
  }

  /// Provide this if the lex is a re-lex.
  // TODO: example
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = Some(re_lex);
    self
  }
}
