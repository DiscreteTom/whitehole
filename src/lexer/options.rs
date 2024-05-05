use super::expectation::Expectation;
use std::marker::PhantomData;

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

/// Pass this to a [`LexOptions`] to enable re-lex.
pub struct ReLexable<ActionState> {
  /// If [`Some`], this will override [`Lexer::action_state`](crate::lexer::Lexer::action_state).
  /// This will be [`Some`] if the re-lexable lex mutated the action state.
  pub action_state: Option<ActionState>,
  pub ctx: ReLexContext,
}

/// See [`LexOptions::fork`].
// we use this trait and 2 structs instead of a `bool` to implement the `Fork` feature
// so that we can return different types in `into_re_lexable` to avoid unnecessary allocations
pub trait LexOptionsFork<ActionState>: Default {
  type ReLexableType: Default;

  fn before_mutate_action_state(&mut self, action_state: &ActionState);
  fn into_re_lexable(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::ReLexableType;
}
pub struct ForkEnabled<ActionState: Clone> {
  /// The action state before any mutation in the current lex.
  action_state_bk: Option<ActionState>,
}
impl<ActionState: Clone> Default for ForkEnabled<ActionState> {
  fn default() -> Self {
    Self {
      action_state_bk: None,
    }
  }
}
impl<ActionState: Clone> LexOptionsFork<ActionState> for ForkEnabled<ActionState> {
  type ReLexableType = Option<ReLexable<ActionState>>;

  fn before_mutate_action_state(&mut self, action_state: &ActionState) {
    if self.action_state_bk.is_none() {
      self.action_state_bk = Some(action_state.clone());
    }
  }

  fn into_re_lexable(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::ReLexableType {
    if action_index < actions_len - 1 {
      // current action is not the last one
      // so the lex is re-lex-able
      Some(ReLexable {
        ctx: ReLexContext {
          skip: action_index + 1, // index + 1 is the count of actions to skip
          start,
        },
        action_state: self.action_state_bk,
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
impl<ActionState> LexOptionsFork<ActionState> for ForkDisabled {
  type ReLexableType = ();

  fn before_mutate_action_state(&mut self, _action_state: &ActionState) {}

  fn into_re_lexable(
    self,
    _start: usize,
    _actions_len: usize,
    _action_index: usize,
  ) -> Self::ReLexableType {
    ()
  }
}

pub struct LexOptions<'expect_text, Kind: 'static, ActionState, Fork: LexOptionsFork<ActionState>> {
  pub expectation: Expectation<'expect_text, Kind>,
  /// See [`LexOptions::fork()`].
  pub fork: Fork,
  /// See [`LexOptions::re_lex()`].
  pub re_lex: Option<ReLexContext>,
  _action_state: PhantomData<ActionState>,
}

impl<'expect_text, Kind, ActionState> Default
  for LexOptions<'expect_text, Kind, ActionState, ForkDisabled>
{
  fn default() -> Self {
    Self {
      expectation: Expectation::default(),
      fork: ForkDisabled,
      re_lex: None,
      _action_state: PhantomData,
    }
  }
}

impl<'expect_text, Kind, ActionState> From<Expectation<'expect_text, Kind>>
  for LexOptions<'expect_text, Kind, ActionState, ForkDisabled>
{
  fn from(expectation: Expectation<'expect_text, Kind>) -> Self {
    Self::default().expect(expectation)
  }
}

impl<'expect_text, Kind, ActionState> From<ReLexContext>
  for LexOptions<'expect_text, Kind, ActionState, ForkDisabled>
{
  fn from(re_lex: ReLexContext) -> Self {
    Self::default().re_lex(re_lex)
  }
}

impl<'expect_text, Kind, ActionState, Fork: LexOptionsFork<ActionState>> From<Fork>
  for LexOptions<'expect_text, Kind, ActionState, Fork>
{
  fn from(fork: Fork) -> Self {
    Self {
      expectation: Expectation::default(),
      fork,
      re_lex: None,
      _action_state: PhantomData,
    }
  }
}

impl<'expect_text, Kind, ActionState, Fork: LexOptionsFork<ActionState>>
  LexOptions<'expect_text, Kind, ActionState, Fork>
{
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_text, Kind>>) -> Self {
    self.expectation = expectation.into();
    self
  }

  /// If set, the [`LexOutput::re_lex`](crate::lexer::output::LexOutput::re_lex) *might* be `Some`.
  // TODO: example
  pub fn fork(self) -> LexOptions<'expect_text, Kind, ActionState, ForkEnabled<ActionState>>
  where
    ActionState: Clone,
  {
    LexOptions {
      expectation: self.expectation,
      fork: ForkEnabled::default(),
      re_lex: self.re_lex,
      _action_state: PhantomData,
    }
  }

  /// Provide this if the lex is a re-lex.
  // TODO: example
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = Some(re_lex);
    self
  }
}
