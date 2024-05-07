use super::{
  expectation::Expectation,
  re_lex::{ReLexContext, ReLexable},
  state::LexerState,
};

/// See [`LexOptions::fork`](crate::lexer::options::LexOptions::fork).
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
