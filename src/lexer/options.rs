use super::{expectation::Expectation, state::LexerState};

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

pub struct ReLexable<'text, 'expect_text, Kind: 'static, ActionState> {
  /// The re-lexable lex's expectation.
  pub expectation: Expectation<'expect_text, Kind>,
  /// If [`Some`], this will override [`Lexer::action_state`](crate::lexer::Lexer::action_state).
  /// This will be [`Some`] if the re-lexable lex mutated the action state.
  /// This will be ignored by [`StatelessLexer`](crate::lexer::stateless::StatelessLexer).
  pub action_state: Option<ActionState>,
  pub lexer_state: LexerState<'text>, // TODO: should this use LexerState or digested+text?
  pub ctx: ReLexContext,
}

/// See [`LexOptions::fork`].
// we use this trait and 2 structs instead of a `bool` to implement the `Fork` feature
// so that we can return different types in `into_re_lexable` to avoid unnecessary allocations
pub trait LexOptionsFork<'text, 'expect_text, Kind: 'static, ActionState>: Default {
  type ReLexableType: Default;

  fn before_mutate_action_state(&mut self, action_state: &ActionState);
  fn into_re_lexable(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
    expectation: Expectation<'expect_text, Kind>,
    lex_start: usize,
    text: &'text str,
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
impl<'text, 'expect_text, Kind: 'static, ActionState: Clone>
  LexOptionsFork<'text, 'expect_text, Kind, ActionState> for ForkEnabled<ActionState>
{
  type ReLexableType = Option<ReLexable<'text, 'expect_text, Kind, ActionState>>;

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
    expectation: Expectation<'expect_text, Kind>,
    lex_start: usize,
    text: &'text str,
  ) -> Self::ReLexableType {
    if action_index < actions_len - 1 {
      let mut lexer_state = LexerState::new(text);
      lexer_state.digest(lex_start); // TODO: optimize this

      // current action is not the last one
      // so the lex is re-lex-able
      Some(ReLexable {
        ctx: ReLexContext {
          skip: action_index + 1, // index + 1 is the count of actions to skip
          start,
        },
        action_state: self.action_state_bk,
        expectation,
        lexer_state,
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
impl<'text, 'expect_text, Kind: 'static, ActionState>
  LexOptionsFork<'text, 'expect_text, Kind, ActionState> for ForkDisabled
{
  type ReLexableType = ();

  fn before_mutate_action_state(&mut self, _action_state: &ActionState) {}

  fn into_re_lexable(
    self,
    _start: usize,
    _actions_len: usize,
    _action_index: usize,
    _expectation: Expectation<'expect_text, Kind>,
    _lex_start: usize,
    _text: &'text str,
  ) -> Self::ReLexableType {
    ()
  }
}

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
